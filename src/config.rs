use std::str::FromStr;

use http::Uri;

#[derive(Clone)]
pub struct XtbClientConfig {
    pub user_id: String,
    pub password: String,
    pub protocol: XtbSupportedProtocol,
    pub host: String,
    pub api_endpoint: String,
    pub api_port: Option<u32>,
    pub stream_api_endpoint: String,
    pub stream_api_port: Option<u32>,
}


impl XtbClientConfig {
    pub fn new_real() -> Self {
        Self::default()
    }

    pub fn new_demo() -> Self {
        Self {
            api_endpoint: "/demo".to_owned(),
            stream_api_endpoint: "/demoStream".to_owned(),
            ..Self::default()
        }
    }

    pub fn make_api_connection_spec(&self) -> ConnectionSpec {
        ConnectionSpec {
            protocol: self.protocol.clone(),
            host: self.host.clone(),
            port: self.api_port.clone(),
            endpoint: self.api_endpoint.clone()
        }
    }

    pub fn make_stream_api_connection_spec(&self) -> ConnectionSpec {
        ConnectionSpec {
            protocol: self.protocol.clone(),
            host: self.host.clone(),
            port: self.stream_api_port.clone(),
            endpoint: self.stream_api_endpoint.clone()
        }
    }
}


impl Default for XtbClientConfig {
    fn default() -> Self {
        Self {
            user_id: String::default(),
            password: String::default(),
            protocol: XtbSupportedProtocol::WSS,
            host: "ws.xtb.com".to_owned(),
            api_endpoint: "/real".to_owned(),
            api_port: None,
            stream_api_endpoint: "/realStream".to_owned(),
            stream_api_port: None,
        }
    }
}


#[derive(Clone)]
pub struct ConnectionSpec {
    pub protocol: XtbSupportedProtocol,
    pub host: String,
    pub endpoint: String,
    pub port: Option<u32>,
}

impl Into<Uri> for ConnectionSpec {
    fn into(self) -> Uri {
        let mut raw_url = format!("{}://{}", self.protocol.to_string(), self.host);
        if let Some(port) = self.port {
            raw_url = format!("{}:{}", raw_url, port);
        }
        raw_url = format!("{}{}", raw_url, self.endpoint);
        Uri::from_str(&raw_url).unwrap()
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
