/// Configuration of the client
#[derive(Clone)]
pub struct XtbClientBuilder {
    pub protocol: XtbSupportedProtocol,
    pub host: String,
    pub api_endpoint: String,
    pub api_port: Option<u32>,
    pub stream_api_endpoint: String,
    pub stream_api_port: Option<u32>,
}


impl XtbClientBuilder {
    pub fn new_real() -> Self {
        Self {
            protocol: XtbSupportedProtocol::WSS,
            host: "ws.xtb.com".to_owned(),
            api_endpoint: "/real".to_owned(),
            api_port: None,
            stream_api_endpoint: "/realStream".to_owned(),
            stream_api_port: None,
        }
    }

    pub fn new_demo() -> Self {
        Self {
            api_endpoint: "/demo".to_owned(),
            stream_api_endpoint: "/demoStream".to_owned(),
            ..Self::default()
        }
    }
}


impl Default for XtbClientBuilder {
    fn default() -> Self {
        Self::new_real()
    }
}


#[derive(Clone)]
pub enum XtbSupportedProtocol {
    WSS,
}


impl ToString for XtbSupportedProtocol {
    fn to_string(&self) -> String {
        match self {
            Self::WSS => "wss"
        }.to_owned()
    }
}
