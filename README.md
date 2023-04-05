# Dynamic SSL Proxy using Rust

SSL proxy in rust using dynamic SSL termination (IE: loading SSL certs at runtime).
Certificates are loaded from a directory based on domain name and cached in memory.


## Generate test certs & hosts file

Example has a script to generate self-signed CA, certs and hosts file data for a handful of test domains.

```shell
# if you have openssl installed

make generate-certs

# use a docker container with openssl
make docker-generate-certs

```

Certs will be saved to `certs/ssl`, the example is coded to load certs from this directory.   

```shell

# add self-signed ca to trusted roots

# on ubuntu
sudo cp certs/ssl/ca.pem /usr/local/share/ca-certificates/rust-ssl-proxy-ca.crt
sudo update-ca-trust

# to remove, delete the certs and update trust
sudo rm -f /usr/local/share/ca-certificates/rust-ssl-proxy-ca.crt
sudo update-ca-trust

```

`certs/hosts` will contain a host entry for all test domains we generated certs for, add the contents to `/etc/hosts`

## Run the example

Open 3 separate terminal windows.

### Terminal 1
```shell
# use an nginx server for proxy destination on port 8080
docker run --rm -it -p 8080:80 nginx:alpine
```
### Terminal 2
```shell
# start the ssl-proxy on port 8443
cargo run --bin ssl-proxy -- 0.0.0.0:8443

```

### Terminal 3
```shell
# execute ssl curl commands to ssl-proxy

curl -i -v https://ssl-1.domain.com:8443
curl -i -v https://ssl-2.domain.com:8443
# ...
curl -i -v https://ssl-10.domain.com:8443


curl -i -v https://ssl-1.domain.net:8443
curl -i -v https://ssl-2.domain.net:8443
# ...
curl -i -v https://ssl-10.domain.net:8443



```
