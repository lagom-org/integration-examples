table env {
    "LAGOM_SECRET": "04b3623a9f7c553c272e3d3def949e3ac781ff8145ee87f22defc7616dae3f86a165547706f5e381a4d70070b234109fdd8daf80167e673ceda05503eb0d3123"
}


sub vcl_recv {
    #FASTLY recv

    # e.g. with the secret above, the following URLs is valid
    # the timestamp is of course past 10 seconds, but the signature is valid.
    # set req.url = "http://localhost:8080/article.html?lgid=lgdp01RKlMjtMghVcMqelB5G&lguid=lgui01RKlLE3o882NZMvPz2CG&lgts=1710325447&lgamt=100&lgsig=479b491b216dea83a0bb8ce9cd3a1a222e9b5ca63abba9400fc384590f193868";

    declare local var.lgid STRING;
    declare local var.lgts STRING;
    declare local var.lgsig STRING;
    declare local var.lguid STRING;
    declare local var.lgamt STRING;
    set var.lgid = querystring.get(req.url, "lgid");
    set var.lgts = querystring.get(req.url, "lgts");
    set var.lguid = querystring.get(req.url, "lguid");
    set var.lgamt = querystring.get(req.url, "lgamt");
    set var.lgsig = "0x"+ querystring.get(req.url, "lgsig");

    # check timestamp, give 10s leeway
    declare local var.lgtsWithClearance TIME;
    set var.lgtsWithClearance = std.time(var.lgts, std.integer2time(-1));
    set var.lgtsWithClearance = time.add(var.lgtsWithClearance, 10s);
    if (time.is_after(now, var.lgtsWithClearance)) {
        error 400 "This link has expired";
    }

    if (var.lgamt != "100") {
        error 400 "This link is not valid";
    }

    # verify signature with pre-shared secret, also check path
    declare local var.sigComputed STRING;
    set var.sigComputed = digest.hmac_sha256(table.lookup(env, "LAGOM_SECRET"), var.lguid + var.lgid + var.lgts + req.url.path + var.lgamt);

    # compare signatures
    if (var.sigComputed != var.lgsig) {
        error 400 "This link is not valid";
    }

    # we are ok !
}
