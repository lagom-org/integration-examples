
python example-python::
	python3 example-python/server.py

fastly-rust example-fastly-rust::
	cd example-fastly-rust && fastly compute serve --addr="127.0.0.1:8080"

fastly-publish deploy::
	cp -r public public-kup
	sed -i "s/http:\/\/localhost:1313/https:\/\/lagom.org/g"  public/article.html
	sed -i "s/http:\/\/localhost:1313/https:\/\/lagom.org/g"  public/full/article.html
	cd example-fastly-rust && fastly compute publish
	rm -rf public
	mv public-kup public

cloudflare-worker example-cloudflare-worker::
	cd example-cloudflare-worker && npx wrangler dev --port 8080

go golang::
	cd example-go/ && go run main.go

watch-python::
	git ls-files | entr -rc make python