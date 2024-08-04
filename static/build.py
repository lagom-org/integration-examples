import os
import sys
import glob

content_tmpl = open("./main.tmpl", "r").read()
aside = open("./aside.tmpl", "r").read()

if os.environ.get("LOGIN"):
	print("Using login aside")
	aside = open("./aside-login.tmpl", "r").read()

content_tmpl = content_tmpl.replace("ASIDE_CONTENT", aside)

for file in glob.iglob('**/*.html', recursive=True):
	print("Processing "	+ file)
	content = open(file, "r").read()
	content = content_tmpl.replace("MAIN_CONTENT", content)
	if os.environ.get("RELEASE"):
		print("Replacing localhost links with lagom.org")
		content = content.replace("http://localhost:1313", "https://lagom.org")
	open(file, "w").write(content)

