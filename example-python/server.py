import http.server
import socketserver
import time
import hmac
import hashlib
import urllib.parse as urlparse

PORT = 8080
HOST = 'localhost'
LAGOM_HOST = 'localhost:3000'

SECRET = '04b3623a9f7c553c272e3d3def949e3ac781ff8145ee87f22defc7616dae3f86a165547706f5e381a4d70070b234109fdd8daf80167e673ceda05503eb0d3123'

TEST_USER_ANONYMIZED_ID = '4N9H26nDNCcTlfrUu5VybrjTiEzDnT1eCB0T0q2GXdX'

class MyServer(http.server.SimpleHTTPRequestHandler):
	def do_GET(self):
		if '/full/' in self.path:
			self.send_response(400)
			self.end_headers()
			self.wfile.write(b'You are not allowed to access this file')
			return

		if 'lgamt' in self.path and self.do_lagom_verif(100):
			self.path = './public/full' + self.path
		else:
			self.path = './public' + self.path

		return http.server.SimpleHTTPRequestHandler.do_GET(self)

	def do_lagom_verif(self, amount):
		# extract params from url, decode and parse
		parsed = urlparse.urlparse(self.path)
		params = urlparse.parse_qs(parsed.query)
		uid = params['lguid'][0]
		ts = params['lgts'][0]
		sig = params['lgsig'][0]
		id = params['lgid'][0]
		amt = params['lgamt'][0]

		# check timestamp is within 10 seconds
		current_time = int(time.time())
		if current_time > int(ts) + 5:
			return False

		# check amount and path
		if int(amt) != amount:
			return False

		# check signature - we also verify that the payment only applies to this page
		verif = uid.encode('utf-8') + id.encode('utf-8') + ts.encode('utf-8') + parsed.path.encode('utf-8') + amt.encode('utf-8')
		good = hmac.new(SECRET.encode('utf-8'), verif, hashlib.sha256).hexdigest()
		if sig != good:
			return False

		return True

	def end_headers(self):
		self.send_header("Cache-Control", "no-cache, no-store, must-revalidate")
		super().end_headers()

socketserver.TCPServer.allow_reuse_address=True
httpd = socketserver.TCPServer((HOST, PORT), MyServer)
print("serving at port", PORT)
httpd.serve_forever()

