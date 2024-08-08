# Score tracker

## Setup

Need to be able to compile to linus musl, so
`rustup target add x86_64-unknown-linux-musl` and
`brew install filosottile/musl-cross/musl-cross` and add the following to
`~/.cargo/config.toml`:

```toml
[target.x86_64-unknown-linux-musl]
linker = "x86_64-linux-musl-gcc"
```

For frontend, `pnpm install`.

## Run

Server: `cargo run -- --config config.json`.

To have live css updates,
`npx tailwindcss -i ./input.css -o ./pages/index.css --watch`.

To see logs of a container, `podman logs <id> [--follow]`.

## Build

- `cargo build --release --target=x86_64-unknown-linux-musl`
- `npx tailwindcss -i ./input.css -o ./pages/index.css --minify`
- `docker build . -f release.dockerfile -t rust-score-tracker`

Then to run the built image:

- `docker run -d -p 127.0.0.1:8000:80 rust-score-tracker --config config.json`.

## Deploy

Save latest image as a .tar file: `docker save -o image.tar rust-score-tracker`.

Copy that to the server: `scp image.tar user@server:/home/user/image.tar`.

Set up the config and copy that to the server:
`scp release.config.json user@server:/home/user/config.json`

Stop and remove the old image `podman ps`, `podman rm -f <id>`,
`podman image rm rust-score-tracker`

Add it to the images on the server: `podman load -i image.tar`.

Run it:

```bash
podman run -d -p 8000:80 -p 8001:443 \
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
Just missing the non-su user.

- Change ssh-port done!
- Setting up firewalld, since debian wiki recommends it
- Shot myself in the foot. Enabled the firewall and then lost the connection,
  now I can't access ssh :facepalm:. Changed the ssh back by booting in rescue
  mode, hope I can connect now.
- It worked! Now really setting up the firewall.
- Done. Followed this guide to set up firewalld
  https://docs.rockylinux.org/guides/security/firewalld-beginners/.
- Changed the port again.
- Set up fail2ban too.
- Now setting up non-su user.
- Not simple. `sudo useradd <name>`,
  `sudo usermod -aG access-certificates <name>`, copy just the right files into
  the home dir. `su - <name>` to login in new shell, otherwise podman won't
  work - and still doesn't. Will look further into this. Probably need to setup
  that the data dir is shared between them.

Would be nice if the app itself knew how to create the missing data.

Also https://wiki.debian.org/Docker - maybe have a look at podman instead?

- Setting up podman to be able to run as non-root.
- Had to set up port forwarding in firewalld because podman can't access 80
  and 443.
  - `sudo firewall-cmd --zone=public --add-masquerade`
  - `sudo firewall-cmd --zone=public --add-forward-port=port=80:proto=tcp:toport=8000`
  - `sudo firewall-cmd --zone=public --add-forward-port=port=443:proto=tcp:toport=8001`
  - test and then `sudo firewall-cmd --runtime-to-permanent`
- Getting errors because I need the certificates from `/etc/letsencrypt` which
  are all root-owned. Found this great article
  https://www.redhat.com/sysadmin/container-permission-denied-errors.
- Changing the owner of `/etc/letsencrypt` recursively worked. Now running
  podman! Curious if this causes problems next time I have to get new
  certificates. But I'll worry about that then.

Followed [the let's encrypt guide](https://letsencrypt.org/getting-started/) to
add tls, since .dev domains have to be https (who knew?). I'm using certbot,
which is running on
[snapd](https://snapcraft.io/docs/installing-snap-on-debian).

Should look into podman quadlets to make the container start again after a
reboot.

Connect to a running docker container:

- `podman ps` to find the container id.
- `podman exec -it <id> bash
