use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::RwLock;
use std::fs::File;
use std::io::{BufReader, Seek};
use std::path::Path;
use std::io;

use rustls_pemfile::{certs, pkcs8_private_keys};

use tokio_rustls::rustls::{Certificate, PrivateKey};
use tokio_rustls::rustls::server::{ClientHello, ResolvesServerCert};
use tokio_rustls::rustls::sign::{any_supported_type, CertifiedKey};


pub struct CertCache(RwLock<HashMap<String, Arc<CertifiedKey>>>);

impl CertCache {
    fn new() -> Self{
        Self(RwLock::new(HashMap::new()))
    }
}

impl CertCache {

    /// save CertifiedKey to cache
    pub fn set(&self, key: String, cert: Arc<CertifiedKey>) {
        let mut c = self.0.write().expect("unable to obtain write lock: cert cache");
        c.deref_mut().insert(key, cert);
    }

    /// retrieve `CertifiedKey` from cache
    pub fn get(&self, key: &String) -> Option<Arc<CertifiedKey>> {

        let guard = match self.0.read() {
            Ok(g) => g,
            _ => {
                return None
            }
        };

        match guard.get(key) {
            Some(c) => {
                Some(Arc::clone(&c))
            },
            _ => None
        }
    }
}

pub struct ResolveSSL {
    pub cache: CertCache
}


impl ResolveSSL {
    pub fn new() -> Self {
        Self{
            cache: CertCache::new()
        }
    }
}

#[test]
fn test_resolvessl_get() {
    // setup test cert pathing
    let domain = "1.ssl-jhardy.com";

    let resolver = ResolveSSL::new();

    // save cert to cache
    let cert = load_certified_key(domain).unwrap();
}


impl ResolvesServerCert for ResolveSSL {
    fn resolve(&self, client_hello: ClientHello) -> Option<Arc<CertifiedKey>> {
        if let Some(domain) = client_hello.server_name() {

            let domain_key = domain.to_string();

            // check cache for cert and return if present
            match self.cache.get(&domain_key) {
                Some(c) => {
                    eprintln!("using ssl cache");
                    return Some(c)
                },
                _ => {
                    eprintln!("ssl cache not found")
                }
            }

            // try and load from file system and save to cache
            return match load_certified_key(domain) {
                Some(c) => {

                    // create arc cert
                    let ac = Arc::new(c);

                    // store cert in cache
                    self.cache.set(domain.to_string(), Arc::clone(&ac));

                    return Some(ac)
                },
                _ => {
                    None
                }
            }
        }
        None
    }
}


fn load_certs(buf: &mut BufReader<File>) -> io::Result<Vec<Certificate>> {
    certs(buf)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid cert"))
        .map(|mut certs| certs.drain(..).map(Certificate).collect())
}

fn load_keys(buf: &mut BufReader<File>) -> io::Result<Vec<PrivateKey>> {
    // rsa_private_keys(buf)
    pkcs8_private_keys(buf)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid key"))
        .map(|mut keys| keys.drain(..).map(PrivateKey).collect())
}

fn load_certified_key(domain: &str) -> Option<CertifiedKey> {

    let file_path = format!("certs/ssl/{}.both.pem", domain);
    let cert_path = Path::new(file_path.as_str());
    let mut cert_buffer = BufReader::new(File::open(cert_path).unwrap());

    eprintln!("cert loaded from file: {}", domain);

    // read in certs to vec
    let certs = match load_certs(&mut cert_buffer) {
        Ok(c) => c,
        _ => {
            eprintln!("cert not found: {}", domain);
            return None
        }
    };

    // rewind buffer before key read
    cert_buffer.rewind().expect("unable to rewind ssl buffer");

    // pull vec of PrivateKey from buffer
    let keys = match load_keys(&mut cert_buffer) {
       Ok(k) => k,
        _ => {
            eprintln!("key not found: {}", domain);
            return None
        }
    };

    let signing_key = match any_supported_type(keys.get(0).unwrap()) {
        Ok(s) => s,
        _ => {
            eprintln!("unable get signing key: {}", domain);
            return None
        }
    };

    Some(CertifiedKey::new(certs, signing_key))
}

/*
pub async fn parse_incoming(stream: &mut TlsStream<TcpStream>) {
    let mut headers = [httparse::EMPTY_HEADER; 32];
    let mut request = httparse::Request::new(&mut headers);

    // buffer request
    let mut buffer = [0u8; 1500];
    let mut result: Vec<u8> = vec![];
    match stream.read(&mut buffer).await {
        Ok(n) => {
            eprintln!("Read: {}", n);
        },
        _ => {
            eprintln!("read error");
        }

    }

    let n = request.parse(&buffer).unwrap();

    if n.is_partial() {
        eprintln!("is partial");
    } else {

    eprintln!("Request: {:?}", request);
    }
    // eprintln!("TlsStream: {:?}", stream);
    // eprintln!("Request Result: {}", result.len());

}
*/
