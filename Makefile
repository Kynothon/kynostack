OPENFAAS_URL?=http://localhost:31112
PLATFORMS:=linux/amd64,linux/arm64,linux/arm/v7
REGISTRY=registry.hub.docker.com
VERSION?=latest
DOCKER_USERNAME?=""
FAAS_PORT?=8080


export DOCKER_CLI_EXPERIMENTAL=enabled

fname = $(strip $(subst $(firstword $(subst -, ,$1))-, , $1))

init:
	docker buildx create --use --name build --node build --driver-opt network=host --platform $(PLATFORMS)

all-%: generate-% build-% deploy-%
	@echo faas-cli invoke -g $(OPENFAAS_URL) $(call fname, $@)

test-%:	generate-% testbuild-%
	@docker run --rm -it -p $(FAAS_PORT):$(FAAS_PORT) $(TAG)

generate-%: 
	$(eval function:=$(call fname, $@))
	@faas-cli build --shrinkwrap --image $(function) -f $(function).yml

build-%: tag-%
	@cd build/$(call fname, $@) && \
		docker buildx build --platform $(PLATFORMS) -t $(TAG) --push .

testbuild-%: tag-%
	cd build/$(call fname, $@) && \
		docker buildx build -t $(TAG) --load .

deploy-%: tag-%
	$(eval function:=$(call fname, $@))
	faas-cli deploy --name=$(function) --image=$(TAG) -g $(OPENFAAS_URL)

tag-%:
ifndef TAG
	$(eval TAG:=$(DOCKER_USERNAME)/$(call fname, $@)-faas:$(VERSION))
endif
	@echo "tag: $(TAG)"


.PHONY: deploy-% build-% generate-% all-% tag-%
