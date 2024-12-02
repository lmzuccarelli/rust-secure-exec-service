use clap::{Parser, Subcommand};
use serde_derive::{Deserialize, Serialize};

/// rust-secure-exec-service cli struct
#[derive(Parser)]
#[command(name = "secure-exec-service")]
#[command(author = "Luigi Mario Zuccarelli <luzuccar@redhat.com>")]
#[command(version = "0.1.0")]
#[command(about = "A simple secure (HTTPS) service that execute commands asynchrously", long_about = None)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    /// set the loglevel
    #[arg(
        value_enum,
        short,
        long,
        value_name = "loglevel",
        default_value = "info",
        help = "Set the log level [possible values: info, debug, trace]"
    )]
    pub loglevel: Option<String>,

    /// set the mode (client or server)
    #[arg(
        short,
        long,
        value_name = "mode",
        help = "Set the mode [possible values: controller, worker] (required)"
    )]
    pub mode: Option<String>,

    /// server ip address (only for worker)
    #[arg(
        short,
        long,
        value_name = "server-ip",
        help = "The server ip address for the worker to connect to (default 127.0.0.1)"
    )]
    pub server_ip: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// RemoteExecute
    RemoteExecute {
        #[arg(
            short,
            long,
            value_name = "node",
            help = "Deploy to a specific node (hostname of server) or all servers"
        )]
        node: String,
    },
    /// RemoteUpload
    RemoteUpload {
        #[arg(
            short,
            long,
            value_name = "node",
            help = "Deploy to a specific node (hostname of server) or all servers"
        )]
        node: String,
        #[arg(
            short,
            long,
            value_name = "file",
            help = "File to upload to remote server"
        )]
        file: String,
    },
}

#[derive(Serialize, Deserialize)]
pub struct BaseConfig {
    #[serde(rename = "created")]
    created: String,

    #[serde(rename = "architecture")]
    architecture: String,

    #[serde(rename = "os")]
    os: String,

    #[serde(rename = "config")]
    config: Config,

    #[serde(rename = "rootfs")]
    rootfs: Rootfs,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "Env")]
    env: Vec<String>,

    #[serde(rename = "Cmd")]
    cmd: Vec<String>,

    #[serde(rename = "Labels")]
    labels: Labels,
}

#[derive(Serialize, Deserialize)]
pub struct Labels {
    #[serde(rename = "architecture")]
    architecture: String,

    #[serde(rename = "build-date")]
    build_date: String,

    #[serde(rename = "component")]
    component: String,

    #[serde(rename = "description")]
    description: String,

    #[serde(rename = "distribution-scope")]
    distribution_scope: String,

    #[serde(rename = "maintainer")]
    maintainer: String,

    #[serde(rename = "name")]
    name: String,

    #[serde(rename = "release")]
    release: String,

    #[serde(rename = "summary")]
    summary: String,

    #[serde(rename = "url")]
    url: String,

    #[serde(rename = "vcs-ref")]
    vcs_ref: String,

    #[serde(rename = "vcs-type")]
    vcs_type: String,

    #[serde(rename = "vendor")]
    vendor: String,

    #[serde(rename = "version")]
    version: String,
}

#[derive(Serialize, Deserialize)]
pub struct Rootfs {
    #[serde(rename = "type")]
    rootfs_type: String,

    #[serde(rename = "diff_ids")]
    diff_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MicroserviceConfig {
    #[serde(rename = "apiVersion")]
    api_version: String,

    #[serde(rename = "kind")]
    kind: String,

    #[serde(rename = "spec")]
    pub spec: Spec,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Spec {
    #[serde(rename = "services")]
    pub services: Vec<Service>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Service {
    /// name must correspond to the actual binary that was created
    #[serde(rename = "name")]
    pub name: String,

    /// binary_path is the path to the actual microservice project on disk
    /// and the link to the binary
    #[serde(rename = "binaryPath")]
    pub binary_path: String,

    /// registry is the oci registry to pull the image from
    #[serde(rename = "registry")]
    pub registry: String,

    #[serde(rename = "version")]
    pub version: String,

    #[serde(rename = "authors")]
    pub authors: Vec<String>,

    #[serde(rename = "description")]
    pub description: String,

    #[serde(rename = "env")]
    pub env: Option<Vec<KeyValue>>,

    #[serde(rename = "args")]
    pub args: Option<Vec<KeyValue>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyValue {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "value")]
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct APIParameters {
    #[serde(rename = "command")]
    pub command: String,

    #[serde(rename = "node")]
    pub node: String,

    #[serde(rename = "user")]
    pub user: String,

    #[serde(rename = "callback")]
    pub callback: bool,

    #[serde(rename = "callbackUrl")]
    pub callback_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct APIResponse {
    #[serde(rename = "status")]
    pub status: String,

    #[serde(rename = "node")]
    pub node: String,

    #[serde(rename = "service")]
    pub service: String,

    #[serde(rename = "text")]
    pub text: String,
}
