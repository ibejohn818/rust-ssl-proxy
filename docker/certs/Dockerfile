FROM alpine:3.16

ENV IS_CONTAINER true

RUN apk add --update --no-cache openssl bash

COPY certs /certs

WORKDIR /certs

ENTRYPOINT ["/certs/generate.sh"]
