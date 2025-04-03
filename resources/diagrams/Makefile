PLANTUML ?= plantuml

.PHONY: all
all: svg

.PHONY: clean
clean: clean-svg

.PHONY: svg
svg: application-deployment-workflow.svg \
     architecture.svg                    \
     device-enrollment-workflow.svg      \
     device-connection-workflow.svg

.PHONY: clean-svg
clean:
	rm *.svg

%.svg: %.puml
	$(PLANTUML) -tsvg $<
