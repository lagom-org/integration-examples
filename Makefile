python example-python:: build-website
	python3 example-python/server.py

watch-python::
	git ls-files | entr -rc make python

fastly-rust example-fastly-rust:: build-website
	cd example-fastly-rust && fastly compute serve --addr="127.0.0.1:8080"

watch-fastly-rust::
	git ls-files | entr -rc make fastly-rust

fastly-full example-fastly-full::
	LOGIN=true make build-website
	cd example-fastly-full && fastly compute serve --addr="127.0.0.1:8080"

watch-fastly-full::
	git ls-files | entr -rc make fastly-full

publish-full::
	LOGIN=true RELEASE=true make build-website
	cd example-fastly-full && fastly compute publish

publish-example::
	RELEASE=true make build-website
	cd example-fastly-rust && fastly compute publish

cloudflare-worker example-cloudflare-worker:: build-website
	cd example-cloudflare-worker && npm run inline && npx wrangler dev --port 8080

go golang:: build-website
	cd example-go/ && go run main.go

build-website::
	rm -rf public
	cp -r static public
	cd public && python build.py