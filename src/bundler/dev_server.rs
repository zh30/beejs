//! Development server module
use anyhow::Result;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
pub struct DevServer {
    port: u16,
    host: String,
}
impl DevServer {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            host: "127.0.0.1".to_string(),
        }
    }
    pub fn start(&self) -> Result<()> {
        println!("Starting dev server on http://{}:{}", self.host, self.port);
        
        // In a real implementation, would start actual HTTP server
        // For now, just simulate
        Ok(())
    }
    pub fn stop(&self) -> Result<()> {
        println!("Stopping dev server");
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
    #[test]
    fn test_dev_server_creation() {
        let server: _ = DevServer::new(3000);
        assert_eq!(server.port, 3000);
    }
}