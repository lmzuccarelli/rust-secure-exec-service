use crate::api::schema::APIParameters;
use crate::httpservices::client::fetch_url;
use ssh2::Session;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufRead;
use std::io::BufReader;
use std::net::TcpStream;
use std::path::Path;
use std::process::{Command, Stdio};
use std::str::FromStr;
//use tokio::sync::Semaphore;

pub async fn remote_execute(api_params: APIParameters) {
    // connect to the local SSH server
    let tcp = TcpStream::connect(api_params.node).unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    sess.userauth_agent(&api_params.user).unwrap();

    let mut channel = sess.channel_session().unwrap();
    channel.exec(&api_params.command).unwrap();
    let mut s = String::new();
    channel.read_to_string(&mut s).unwrap();
    println!("{}", s);
    let _res = channel.wait_close();
    println!("{}", channel.exit_status().unwrap());
    if api_params.callback {
        let url_str = api_params.callback_url.unwrap();
        let uri = hyper::Uri::from_str(&url_str).unwrap();
        let _res = fetch_url(uri).await;
    }
}

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
    //if res_file.is_err() {
    //    let err = MirrorError::new(&format!(
    //        "opening artifact file {}",
    //        res_file.err().unwrap().to_string().to_lowercase()
    //    ));
    //    return Err(err);
    //}
    let res_r = res_file.unwrap().read_to_end(&mut file_buf).unwrap();
    println!("result from read {}", res_r);

    let mut remote_file = sess
        .scp_send(
            Path::new("/home/lzuccarelli/kaka"),
            0o755,
            res_r as u64,
            None,
        )
        .unwrap();
    //remote_file.write(b"1234567890").unwrap();
    // write the file
    let res = remote_file.write_all(&file_buf);
    println!("result from write {:?}", res);
    // Close the channel and wait for the whole content to be transferred
    remote_file.send_eof().unwrap();
    remote_file.wait_eof().unwrap();
    remote_file.close().unwrap();
    remote_file.wait_close().unwrap();
}

pub async fn local_execute(api_params: APIParameters) {
    let cmd = Command::new(api_params.command)
        .stdout(Stdio::piped())
        .spawn()
        .expect("command failed");
    let mut out = cmd.stdout.unwrap();
    let mut reader = BufReader::new(&mut out);

    loop {
        let mut line = String::new();
        let num_bytes = reader.read_line(&mut line).unwrap();
        if num_bytes == 0 {
            println!("End of stream.");
            break;
        }
        println!("The line: {:?}", line);
    }
    println!("Done.");
}
