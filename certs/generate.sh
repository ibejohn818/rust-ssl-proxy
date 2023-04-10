#!/usr/bin/env bash

CA_CN="ssl-proxy.jhardy.com"

HELP="$(basename "$0") [OPTIONS] [ACTION] 

Generate self-signed certs for ssl proxy tests

OPTIONS:
     -h/--help  show this help text
       --ca-cn  CN of self signed ca. Default: ${CA_CN} 
ARGS:
    ACTION  ca | certs | hosts (optional)"

ARG_POS=0
ACTION=""

DIR=$(realpath "$(dirname "${0}")")

while [[ $# -gt 0 ]]; do
  case $1 in
    -h|--help)
      echo "${HELP}"
      exit 0;
      ;;
    --ca-cn)
      CA_CN="${2}"
      shift
      shift
      ;;
    -*|--*)
      echo "Unknown flag $1"
      exit 1
      ;;
    *)
      case "${ARG_POS}" in
        0)
          ACTION="${1}"
          ARG_POS=1
          shift
          ;;
        *)
          echo "Unknown argument: ${1}" 
          echo ""
          echo "${HELP}"
          exit 1;
      esac
      ;;
  esac
done

set -ex


SAVE_TO="${DIR}/ssl"
mkdir "${SAVE_TO}" || true

# check if ca exists, if not generate
if  { [[ -z "${ACTION}" ]] || [[ "${ACTION}" == 'ca' ]]; } && [ ! -f "${SAVE_TO}/ca.pem" ]; then

  echo "Generating CA"

  openssl genpkey -out "${SAVE_TO}/ca.key" -algorithm RSA -pkeyopt rsa_keygen_bits:4096

  openssl req -x509 -new -nodes \
        -key "${SAVE_TO}/ca.key"  \
        -days 1825 -out "${SAVE_TO}/ca.pem" \
         -subj "/C=US/ST=California/L=Los Angeles/CN=${CA_CN}" \

fi


DOMAINS=("domain.com" "domain.net" "domain.org" "domain.dev")
HOSTS=""

# create certs and a host file
for _dom in "${DOMAINS[@]}"; do
  for i in {1..100}; do 

    _domain="ssl-${i}.${_dom}"
    HOSTS="${HOSTS} ${_domain}"

    if [[ -z "${ACTION}" ]] || [[ "${ACTION}" == 'certs' ]];
    then
      key="${_domain}.key"
      cert="${_domain}.pem"
      csr="${_domain}.csr"
      both="${_domain}.both.pem"

      # RSA
      # openssl genrsa -out "${SAVE_TO}/${key}" 4096
      # PKCS8
      openssl genpkey -out "${SAVE_TO}/${key}" -algorithm RSA -pkeyopt rsa_keygen_bits:4096

      openssl req -new -key "${SAVE_TO}/${key}" -out "${SAVE_TO}/${csr}" \
       -subj "/C=US/ST=California/L=Los Angeles/O=jhardy-lab/CN=${_domain}/" \
       -addext "subjectAltName=DNS:${_domain},IP:0.0.0.0"

      openssl x509 -req -days 365 -in "${SAVE_TO}/${csr}" -CA "${SAVE_TO}/ca.pem" \
        -CAkey "${SAVE_TO}/ca.key" -CAcreateserial \
        -extfile <(printf "subjectAltName=DNS:${_domain},IP:0.0.0.0") \
        -out "${SAVE_TO}/${cert}"

      # combine cert + key
      rm -rf "${SAVE_TO:?}/${both}" || true
      cat "${SAVE_TO}/${cert}" > "${SAVE_TO}/${both}"
      cat "${SAVE_TO}/${key}" >> "${SAVE_TO}/${both}"
    fi


  done
done

HOSTS_FILE="${DIR}/hosts"

if [[ -z "${ACTION}" ]] || [[ "${ACTION}" == 'hosts' ]];
then

  rm -rf "${HOSTS_FILE}" || true

  echo "0.0.0.0 ${HOSTS}" > "${HOSTS_FILE}"

  echo "Append to /etc/hosts: ${DIR}/hosts"

fi

if [[ -n "${IS_CONTAINER}" ]];
then
  chmod -R 777 "${SAVE_TO}"
  if [ -f "${HOSTS_FILE}" ];
  then
    chmod 777 "${HOSTS_FILE}"
  fi
fi
