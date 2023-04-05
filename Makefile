PLATFORM ?= "linux/amd64"

.PHONY: docker/certs


docker/certs/build:
	docker buildx build \
		-f docker/certs/Dockerfile \
		--platform $(PLATFORM) \
		-t ssl-proxy/certs .

docker-generate-certs: docker/certs/build
	docker run --rm -it \
		--platform $(PLATFORM) \
		-v $(shell pwd)/certs/ssl:/certs/ssl \
		ssl-proxy/certs

generate-certs:
	./certs/generate.sh
