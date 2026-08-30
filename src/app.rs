//! The application, and the one seam the tests use.
//!
//! An `App` is constructed from config, a clock, and a capture sink, and binds its listeners
//! immediately so a test can learn the ephemeral port it landed on.

use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use crate::capture::CaptureSink;
use crate::clock::Clock;
use crate::config::Config;
use crate::modbus::ModbusSurface;

pub struct App {
    modbus_addr: SocketAddr,
    tasks: Vec<JoinHandle<()>>,
}

impl App {
    /// Bind every surface and start serving. Returns once the ports are bound, so a caller that
    /// asked for port 0 can read back what it got.
    pub async fn bind(
        config: Config,
        clock: Arc<dyn Clock>,
        sink: Arc<dyn CaptureSink>,
    ) -> io::Result<Self> {
        let listener = TcpListener::bind(config.modbus.bind).await?;
        let modbus_addr = listener.local_addr()?;

        let modbus = Arc::new(ModbusSurface::new(&config, clock, sink));
        let task = tokio::spawn(modbus.serve(listener));

        Ok(Self {
            modbus_addr,
            tasks: vec![task],
        })
    }

    pub fn modbus_addr(&self) -> SocketAddr {
        self.modbus_addr
    }

    /// Serve until a surface stops on its own, which in practice means never.
    pub async fn wait(&mut self) {
        for task in self.tasks.drain(..) {
            let _ = task.await;
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        for task in &self.tasks {
            task.abort();
        }
    }
}
