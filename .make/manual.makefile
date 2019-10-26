template = $(eval $(foreach item,$(2),$(call _template__$(1),$(item))))

define _template__xml
target/$(1:%.pro=%.xml): $1 target/release/prosidy
	@mkdir -p target/$(dir $1)
	target/release/prosidy compile $$(PROSIDY_FLAGS) --out $$@ $$<
endef

