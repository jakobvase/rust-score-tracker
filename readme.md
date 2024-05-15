# Score tracker

## Run

`cargo run`.

## Build

Need to be able to compile to linus musl, so
`rustup target add x86_64-unknown-linux-musl` and
`brew install filosottile/musl-cross/musl-cross` and add the following to
`~/.cargo/config.toml`:

```toml
[target.x86_64-unknown-linux-musl]
linker = "x86_64-linux-musl-gcc"
```

Build: `cargo build --release --target=x86_64-unknown-linux-musl`. Then
`docker build . -f release.dockerfile -t rust-score-tracker`. Then to run
`docker run -d -p 127.0.0.1:8000:80 rust-score-tracker --config config.json`.

## Deploy

Save latest image as a .tar file: `docker save -o image.tar rust-score-tracker`.

Copy that to the server: `scp image.tar user@server:/home/user/image.tar`.

Stop and remove the old image `sudo docker ps`, `sudo docker rm -f <id>`,
`sudo docker image rm rust-score-tracker`

Add it to the images on the server: `sudo docker load -i image.tar`.

Run it:

```bash
sudo docker run -d -p 80:80 -p 443:443 \
-v rust-score-tracker-data:/app/data \
-v /home/user/config.json:/app/config.json \
-v "/home/user/score-tracker-static/.well-known/acme-challenge:/app/acme" \
-v "/etc/letsencrypt:/app/certs" \
rust-score-tracker --config /app/config.json
```

## Notes

Used
[this guide](https://www.digitalocean.com/community/tutorials/how-to-install-and-use-docker-on-debian-10)
to install docker on the debian instance.

Also would be interested to follow the
[guides on security from OVH](https://help.ovhcloud.com/csm/en-gb-vps-security-tips?id=kb_article_view&sysparm_article=KB0047706).
Not done yet though. Server up first!

Would be nice if the app itself knew how to create the missing data.

Also https://wiki.debian.org/Docker - maybe have a look at podman instead?

Followed [the let's encrypt guide](https://letsencrypt.org/getting-started/) to
add tls, since .dev domains have to be https (who knew?). I'm using certbot,
which is running on
[snapd](https://snapcraft.io/docs/installing-snap-on-debian).
