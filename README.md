# Lagom Integration Examples

This repo contains all the integration examples for [Lagom](https://lagom.org). We include examples for integrating with various webservers and CDNs. Feel free to explore the examples and use them as a reference for your own projects.

We use this repo to serve our [demo website](https://demo.lagom.org), using Fastly with compute-at-edge as a CDN.

## Running the examples

To run the Golang example simply run
```sh
$ cd example-go/ && go run main.go
```

Have a look at the Makefile to see how to run the other examples.

## User Interface

The UI served for this example repo lives in the `public` folder. This example UI is a simple static news website.