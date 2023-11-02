.PHONY: run all clean

BUILD = debug

all: target/$(BUILD)/main.jar target/$(BUILD)/jni/MainKt.h

clean:
	cargo clean

run: target/$(BUILD)/main.jar
	kotlin -classpath $< MainKt

target/$(BUILD)/liblusque.so: $(shell find src -type f)
ifeq ($(BUILD), release)
	cargo b -r
else
	cargo b
endif

target/$(BUILD)/main.jar: target/$(BUILD)/kotlin/ target/$(BUILD)/liblusque.so
	cd target/$(BUILD)/kotlin/ && cp ../liblusque.so . && jar cf ../main.jar $$(find)

target/$(BUILD)/kotlin/: $(shell find ktsrc -type f)
	@rm -rf $@
	@mkdir -p $@
	kotlinc -d $@ $^

target/$(BUILD)/jni/%.h: target/$(BUILD)/main.jar
	javah -cp $< $*