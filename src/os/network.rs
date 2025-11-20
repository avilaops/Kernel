use std::io::{self, Read, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream, ToSocketAddrs, UdpSocket};
use std::time::Duration;

/// Abstração de TCP server
pub struct TcpServer {
    listener: TcpListener,
}

impl TcpServer {
    /// Cria um servidor TCP e faz bind no endereço
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        Ok(Self { listener })
    }

    /// Define timeout para accept
    pub fn set_accept_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.listener.set_nonblocking(timeout.is_none())
    }

    /// Aceita uma conexão
    pub fn accept(&self) -> io::Result<(TcpClient, SocketAddr)> {
        let (stream, addr) = self.listener.accept()?;
        Ok((TcpClient { stream }, addr))
    }

    /// Retorna o endereço local
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    /// Cria um iterator de conexões
    pub fn incoming(&self) -> impl Iterator<Item = io::Result<(TcpClient, SocketAddr)>> + '_ {
        self.listener.incoming().map(|result| {
            result.map(|stream| {
                let addr = stream.peer_addr().unwrap();
                (TcpClient { stream }, addr)
            })
        })
    }
}

/// Abstração de TCP client
pub struct TcpClient {
    stream: TcpStream,
}

impl TcpClient {
    /// Conecta a um servidor TCP
    pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let stream = TcpStream::connect(addr)?;
        Ok(Self { stream })
    }

    /// Conecta com timeout
    pub fn connect_timeout(addr: &SocketAddr, timeout: Duration) -> io::Result<Self> {
        let stream = TcpStream::connect_timeout(addr, timeout)?;
        Ok(Self { stream })
    }

    /// Define timeout de leitura
    pub fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.stream.set_read_timeout(timeout)
    }

    /// Define timeout de escrita
    pub fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.stream.set_write_timeout(timeout)
    }

    /// Define nodelay (desabilita algoritmo de Nagle)
    pub fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
        self.stream.set_nodelay(nodelay)
    }

    /// Retorna o endereço local
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.stream.local_addr()
    }

    /// Retorna o endereço remoto
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.stream.peer_addr()
    }

    /// Desliga a conexão
    pub fn shutdown(&self, how: std::net::Shutdown) -> io::Result<()> {
        self.stream.shutdown(how)
    }

    /// Envia dados
    pub fn send(&mut self, data: &[u8]) -> io::Result<usize> {
        self.stream.write(data)
    }

    /// Envia todos os dados
    pub fn send_all(&mut self, data: &[u8]) -> io::Result<()> {
        self.stream.write_all(data)
    }

    /// Recebe dados
    pub fn recv(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buffer)
    }

    /// Recebe exatamente N bytes
    pub fn recv_exact(&mut self, buffer: &mut [u8]) -> io::Result<()> {
        self.stream.read_exact(buffer)
    }
}

impl Read for TcpClient {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

impl Write for TcpClient {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

/// Abstração de UDP socket
pub struct UdpClient {
    socket: UdpSocket,
}

impl UdpClient {
    /// Cria um socket UDP e faz bind
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        Ok(Self { socket })
    }

    /// Conecta a um endereço remoto (para envio direto)
    pub fn connect<A: ToSocketAddrs>(&self, addr: A) -> io::Result<()> {
        self.socket.connect(addr)
    }

    /// Envia dados para um endereço
    pub fn send_to<A: ToSocketAddrs>(&self, buf: &[u8], addr: A) -> io::Result<usize> {
        self.socket.send_to(buf, addr)
    }

    /// Envia dados (requer connect prévio)
    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        self.socket.send(buf)
    }

    /// Recebe dados
    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.socket.recv_from(buf)
    }

    /// Recebe dados (requer connect prévio)
    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.socket.recv(buf)
    }

    /// Define timeout de leitura
    pub fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.socket.set_read_timeout(timeout)
    }

    /// Define timeout de escrita
    pub fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.socket.set_write_timeout(timeout)
    }

    /// Define broadcast
    pub fn set_broadcast(&self, broadcast: bool) -> io::Result<()> {
        self.socket.set_broadcast(broadcast)
    }

    /// Retorna o endereço local
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    /// Retorna o endereço remoto (se conectado)
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.socket.peer_addr()
    }
}

/// Network utilities
pub struct Network;

impl Network {
    /// Resolve hostname para endereços IP
    pub fn resolve<A: ToSocketAddrs>(addr: A) -> io::Result<Vec<SocketAddr>> {
        addr.to_socket_addrs().map(|iter| iter.collect())
    }

    /// Verifica se uma porta está disponível
    pub fn is_port_available(port: u16) -> bool {
        TcpListener::bind(("127.0.0.1", port)).is_ok()
    }

    /// Encontra uma porta disponível
    pub fn find_available_port(start: u16, end: u16) -> Option<u16> {
        (start..=end).find(|&port| Self::is_port_available(port))
    }

    /// Retorna o hostname do sistema
    pub fn hostname() -> Option<String> {
        hostname::get().ok().and_then(|h| h.into_string().ok())
    }

    /// Pinga um endereço (TCP connect test)
    pub fn ping(addr: &SocketAddr, timeout: Duration) -> bool {
        TcpStream::connect_timeout(addr, timeout).is_ok()
    }
}

/// HTTP client simples (sem dependências externas)
pub struct HttpClient;

impl HttpClient {
    /// Faz uma requisição HTTP GET simples
    pub fn get(url: &str) -> io::Result<String> {
        // Parse URL simples
        let (host, port, path) = Self::parse_url(url)?;

        // Conecta
        let addr = format!("{}:{}", host, port);
        let mut client = TcpClient::connect(addr)?;

        // Envia requisição
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            path, host
        );
        client.send_all(request.as_bytes())?;

        // Lê resposta
        let mut response = String::new();
        client.stream.read_to_string(&mut response)?;

        Ok(response)
    }

    fn parse_url(url: &str) -> io::Result<(String, u16, String)> {
        let url = url
            .trim_start_matches("http://")
            .trim_start_matches("https://");

        let parts: Vec<&str> = url.splitn(2, '/').collect();
        let host_port = parts[0];
        let path = if parts.len() > 1 {
            format!("/{}", parts[1])
        } else {
            "/".to_string()
        };

        let (host, port) = if host_port.contains(':') {
            let hp: Vec<&str> = host_port.split(':').collect();
            (hp[0].to_string(), hp[1].parse().unwrap_or(80))
        } else {
            (host_port.to_string(), 80)
        };

        Ok((host, port, path))
    }
}

/// Endereço IP utilities
pub struct IpAddress;

impl IpAddress {
    /// Converte string para IpAddr
    pub fn parse(s: &str) -> Option<IpAddr> {
        s.parse().ok()
    }

    /// Verifica se é localhost
    pub fn is_localhost(ip: &IpAddr) -> bool {
        match ip {
            IpAddr::V4(v4) => v4.is_loopback(),
            IpAddr::V6(v6) => v6.is_loopback(),
        }
    }

    /// Verifica se é endereço privado
    pub fn is_private(ip: &IpAddr) -> bool {
        match ip {
            IpAddr::V4(v4) => v4.is_private(),
            IpAddr::V6(_) => false, // Simplificado
        }
    }

    /// Verifica se é multicast
    pub fn is_multicast(ip: &IpAddr) -> bool {
        match ip {
            IpAddr::V4(v4) => v4.is_multicast(),
            IpAddr::V6(v6) => v6.is_multicast(),
        }
    }
}

/// Buffer de rede para construir mensagens
pub struct NetworkBuffer {
    data: Vec<u8>,
}

impl NetworkBuffer {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    pub fn write_u8(&mut self, value: u8) {
        self.data.push(value);
    }

    pub fn write_u16(&mut self, value: u16) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    pub fn write_u32(&mut self, value: u32) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    pub fn write_u64(&mut self, value: u64) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    pub fn write_string(&mut self, s: &str) {
        let bytes = s.as_bytes();
        self.write_u32(bytes.len() as u32);
        self.write_bytes(bytes);
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Default for NetworkBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_local() {
        let server = TcpServer::bind("127.0.0.1:0").unwrap();
        let addr = server.local_addr().unwrap();

        let handle = std::thread::spawn(move || {
            let (mut client, _) = server.accept().unwrap();
            let mut buf = [0u8; 5];
            client.recv_exact(&mut buf).unwrap();
            assert_eq!(&buf, b"hello");
        });

        let mut client = TcpClient::connect(addr).unwrap();
        client.send_all(b"hello").unwrap();

        handle.join().unwrap();
    }

    #[test]
    fn test_udp_local() {
        let server = UdpClient::bind("127.0.0.1:0").unwrap();
        let server_addr = server.local_addr().unwrap();

        let client = UdpClient::bind("127.0.0.1:0").unwrap();
        client.send_to(b"hello", server_addr).unwrap();

        let mut buf = [0u8; 10];
        let (size, _) = server.recv_from(&mut buf).unwrap();
        assert_eq!(&buf[..size], b"hello");
    }

    #[test]
    fn test_network_buffer() {
        let mut buf = NetworkBuffer::new();
        buf.write_u8(42);
        buf.write_u16(1000);
        buf.write_string("test");

        assert!(!buf.is_empty());
        assert!(buf.len() > 0);
    }

    #[test]
    fn test_port_available() {
        // A porta 0 sempre deve estar disponível (sistema aloca)
        assert!(Network::is_port_available(0));
    }
}
