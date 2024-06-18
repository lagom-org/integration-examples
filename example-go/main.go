package main

import (
	"crypto/hmac"
	"crypto/sha256"
	"fmt"
	"net/http"
	"strconv"
	"time"
)

const SECRET = "04b3623a9f7c553c272e3d3def949e3ac781ff8145ee87f22defc7616dae3f86a165547706f5e381a4d70070b234109fdd8daf80167e673ceda05503eb0d3123"

func lagomVerify(req *http.Request, page string, amount int) error {
	// extract callback params from URL, decode and parse
	lguid := req.URL.Query().Get("lguid")
	lgts := req.URL.Query().Get("lgts")
	lgsig := req.URL.Query().Get("lgsig")
	lgid := req.URL.Query().Get("lgid")
	lgamt := req.URL.Query().Get("lgamt")

	// verify timestamp freshness
	ts, err := strconv.Atoi(lgts)
	if err != nil || int(time.Now().UTC().Unix()) > ts+10 {
		return fmt.Errorf("This link has expired")
	}

	// check amount and page
	lgamtint, err := strconv.Atoi(lgamt)
	if err != nil || lgamtint != amount {
		return fmt.Errorf("This link is not valid")
	}

	// verify signature with pre shared secret
	mac := hmac.New(sha256.New, []byte(SECRET))
	mac.Write([]byte(lguid + lgid + lgts + page + lgamt))
	ret := mac.Sum(nil)
	if lgsig != fmt.Sprintf("%x", ret) {
		return fmt.Errorf("This link is not valid")
	}

	return nil
}

func main() {
	http.Handle("/", http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		err := lagomVerify(r, "/", 100)
		if err != nil {
			http.ServeFile(w, r, "../public/article.html")
		} else {
			http.ServeFile(w, r, "../public/full/article.html")
		}
	}))

	fmt.Println("Server is running on port 8080")
	http.ListenAndServe(":8080", nil)
}
