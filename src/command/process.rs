use crate::api::schema::NodeExecute;
use crate::error::handler::TaskExecuteError;
use crate::httpservices::client::fetch_url;
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

pub async fn remote_execute(node: &NodeExecute) -> Result<(), TaskExecuteError> {
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
    let exit = channel.exit_status().unwrap();
    if exit != 0 {
        let url_str = &node.parameters.error_url;
        let uri = hyper::Uri::from_str(url_str.as_ref()).unwrap();
        let _res = fetch_url(uri).await;
        let err = TaskExecuteError::new(&format!(
            "executing remote command  {}",
            node.parameters.command
        ));
        return Err(err);
    }
    if node.parameters.console_log {
        println!("{}", s);
    } else {
        let _ = fs::write(format!("logs/{}.log", node.name.replace(":", "-")), s);
    }
    if node.parameters.callback {
        let url_str = &node.parameters.callback_url;
        let uri = hyper::Uri::from_str(url_str.as_ref().unwrap()).unwrap();
        let _res = fetch_url(uri).await;
    }
    if Path::new("semaphore.pid").exists() {
        fs::remove_file("semaphore.pid").expect("should delete semaphore");
    }
    Ok(())
}

#[allow(unused)]
pub async fn remote_upload(node: String, file: String) {
    // connect to the local SSH server
    let tcp = TcpStream::connect(node.clone()).unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    sess.userauth_agent("lzuccarelli").unwrap();

    // read local file
    let mut file_buf = vec![];
    let res_file = File::open(file.clone());
    let res_r = res_file.unwrap().read_to_end(&mut file_buf).unwrap();
    println!("result from read {}", res_r);

    let mut remote_file = sess
        .scp_send(
            Path::new("/home/lzuccarelli/test"),
            0o755,
            res_r as u64,
            None,
        )
        .unwrap();
    let res = remote_file.write_all(&file_buf);
    println!("result from write {:?}", res);
    // close the channel and wait for the whole content to be transferred
    remote_file.send_eof().unwrap();
    remote_file.wait_eof().unwrap();
    remote_file.close().unwrap();
    remote_file.wait_close().unwrap();
}

pub async fn local_execute(node: &NodeExecute) {
    let mut exit_status = false;
    let cmd = Command::new(&node.parameters.command)
        .stdout(Stdio::piped())
        .spawn();
    if cmd.is_err() {
        println!("{}", cmd.as_ref().err().unwrap());
        let url_str = &node.parameters.error_url;
        let uri = hyper::Uri::from_str(url_str.as_ref()).unwrap();
        let _res = fetch_url(uri).await;
        fs::remove_file("semaphore.pid").expect("should delete semaphore");
    } else {
        let mut out = cmd.unwrap().stdout.unwrap();
        let mut reader = BufReader::new(&mut out);
        if node.parameters.console_log {
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
            let url_str = &node.parameters.error_url;
            let uri = hyper::Uri::from_str(url_str.as_ref()).unwrap();
            let _res = fetch_url(uri).await;
        } else if node.parameters.callback {
            let url_str = &node.parameters.callback_url;
            let uri = hyper::Uri::from_str(url_str.as_ref().unwrap()).unwrap();
            let _res = fetch_url(uri).await;
        }
        if Path::new("semaphore.pid").exists() {
            fs::remove_file("semaphore.pid").expect("should delete semaphore");
        }
    }
}
