use crate::api::schema::FileUpload;
use crate::api::schema::TaskExecute;
use crate::error::handler::TaskExecuteError;
use crate::httpservices::client::fetch_url;
use custom_logger as log;
use ssh2::Session;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufRead;
use std::io::BufReader;
use std::net::TcpStream;
use std::path::Path;
use std::process::{Command, Stdio};
use std::str::FromStr;

pub async fn execute(task_execute: &TaskExecute) -> Result<(), TaskExecuteError> {
    // acquire
    fs::write("semaphore.pid", "process").expect("should write semaphore");
    for node in task_execute.spec.nodes.iter() {
        if node.name != "localhost" {
            // connect to the local SSH server
            let tcp = TcpStream::connect(&node.name).unwrap();
            let mut sess = Session::new().unwrap();
            sess.set_tcp_stream(tcp);
            sess.handshake().unwrap();
            sess.userauth_agent(&node.parameters.user).unwrap();
            let mut channel = sess.channel_session().unwrap();
            channel.exec(&node.parameters.command).unwrap();
            let mut s = String::new();
            channel.read_to_string(&mut s).unwrap();
            let _res = channel.wait_close();
            let exit_status = channel.exit_status().unwrap();
            if exit_status != 0 {
                fs::remove_file("semaphore.pid").expect("should delete semaphore");
                let url_str = &task_execute.spec.error_url;
                let uri = hyper::Uri::from_str(url_str.as_ref().unwrap()).unwrap();
                let _res = fetch_url(uri).await;
                let err = TaskExecuteError::new(&format!(
                    "[execute] (remote) command failed: {} status: {}",
                    node.parameters.command, exit_status,
                ));
                log::error!("[execute] (remote) {:?}", err);
                return Err(err);
            }
            if node.parameters.console_log {
                println!("{}", s);
            } else {
                let _ = fs::write(format!("logs/{}.log", node.name.replace(":", "-")), s);
            }
        } else {
            let mut exit_status = false;
            let mut command = Command::new(&node.parameters.command);
            if node.parameters.args.is_some() {
                for arg in node.parameters.args.as_ref().unwrap().iter() {
                    command.arg(arg);
                }
            }
            let cmd_res = command.stdout(Stdio::piped()).spawn();
            if cmd_res.is_err() {
                fs::remove_file("semaphore.pid").expect("should delete semaphore");
                let url_str = &task_execute.spec.error_url;
                let uri = hyper::Uri::from_str(url_str.as_ref().unwrap()).unwrap();
                let _res = fetch_url(uri).await;
                let err = TaskExecuteError::new(&format!(
                    "[execute] command failed : {}",
                    cmd_res.err().unwrap().to_string().to_lowercase()
                ));
                log::error!("[execute] {:?}", err);
                return Err(err);
            }
            let mut out = cmd_res.unwrap().stdout.unwrap();
            let mut reader = BufReader::new(&mut out);
            if node.parameters.console_log {
                // we uase println and not custom_logger to preserve original output
                loop {
                    let mut line = String::new();
                    let num_bytes = reader.read_line(&mut line).unwrap();
                    if line.contains("exit => 0") {
                        exit_status = true;
                    }
                    if num_bytes == 0 {
                        println!("=> end of stream");
                        break;
                    }
                    // this preserves colors
                    print!("{}", line);
                }
            } else {
                let mut data = String::new();
                let _ = reader.read_to_string(&mut data);
                let _ = fs::write(format!("logs/{}.log", node.name), data);
            }
            if !exit_status {
                fs::remove_file("semaphore.pid").expect("should delete semaphore");
                let url_str = &task_execute.spec.error_url;
                let uri = hyper::Uri::from_str(url_str.as_ref().unwrap()).unwrap();
                let _res = fetch_url(uri).await;
                let err = TaskExecuteError::new(&format!(
                    "[local_execute] command failed : {} status: {}",
                    node.parameters.command, exit_status
                ));
                log::error!("[execute] {:?}", err);
                return Err(err);
            }
        }
    }
    // callback and drop
    if task_execute.spec.callback {
        let url_str = &task_execute.spec.callback_url;
        let uri = hyper::Uri::from_str(url_str.as_ref().unwrap()).unwrap();
        let _res = fetch_url(uri).await;
    }
    fs::remove_file("semaphore.pid").expect("should delete semaphore");
    Ok(())
}

pub async fn remote_upload(file_upload: &FileUpload) -> Result<(), TaskExecuteError> {
    // acquire
    fs::write("semaphore.pid", "process").expect("should write semaphore");
    for node in file_upload.spec.nodes.iter() {
        // connect to the local SSH server
        let tcp = TcpStream::connect(node.name.clone()).unwrap();
        let mut sess = Session::new().unwrap();
        sess.set_tcp_stream(tcp);
        sess.handshake().unwrap();
        sess.userauth_agent(&node.user).unwrap();

        // read local file
        let mut file_buf = vec![];
        let res_file = File::open(node.file.clone());
        let res_r = res_file.unwrap().read_to_end(&mut file_buf);
        if res_r.is_err() {
            fs::remove_file("semaphore.pid").expect("should delete semaphore");
            let url_str = &file_upload.spec.error_url;
            let uri = hyper::Uri::from_str(url_str.as_ref().unwrap()).unwrap();
            let _res = fetch_url(uri).await;
            let err = TaskExecuteError::new(&format!(
                "[remote_upload] failed: {}",
                res_r.err().unwrap().to_string().to_lowercase()
            ));
            return Err(err);
        }

        let mode = match node.mode.as_str() {
            "0755" => 0o755,
            "0766" => 0o766,
            _ => 0o644,
        };
        let mut remote_file = sess
            .scp_send(Path::new(&node.path), mode, res_r.unwrap() as u64, None)
            .unwrap();
        let res = remote_file.write_all(&file_buf.clone());
        if res.is_err() {
            fs::remove_file("semaphore.pid").expect("should delete semaphore");
            let url_str = &file_upload.spec.error_url;
            let uri = hyper::Uri::from_str(url_str.as_ref().unwrap()).unwrap();
            let _res = fetch_url(uri).await;
            let err = TaskExecuteError::new(&format!(
                "[remote_upload] failed: {}",
                res.err().unwrap().to_string().to_lowercase()
            ));
            return Err(err);
        }
        // close the channel and wait for the whole content to be transferred
        remote_file.send_eof().unwrap();
        remote_file.wait_eof().unwrap();
        remote_file.close().unwrap();
        remote_file.wait_close().unwrap();
    }
    // callback and drop
    if file_upload.spec.callback {
        let url_str = &file_upload.spec.callback_url;
        let uri = hyper::Uri::from_str(url_str.as_ref().unwrap()).unwrap();
        let _res = fetch_url(uri).await;
    }
    fs::remove_file("semaphore.pid").expect("should delete semaphore");
    Ok(())
}
