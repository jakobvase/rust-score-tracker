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

Stop and remove the old image `podman stop rust_score_tracker_server`,
`podman rm rust_score_tracker_server`, `podman image rm rust-score-tracker`

Add it to the images on the server: `podman load -i image.tar`.

Run it:

```bash
podman run -d -p 8000:80 -p 8001:443 \
--name rust_score_tracker_server \
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

### Podman

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
- It did cause problems. Permissions denied.
- Did a lot of searching, and ended up on
  https://www.redhat.com/sysadmin/supplemental-groups-podman-containers, where I
  finally found something that worked. Added the following to
  `~/.config/containers/containers.conf`:
  ```
  [containers]
  annotations=["run.oci.keep_original_groups=1",]
  ```
  which keeps the groups I added, and certbot copies the groups to new
  certificates. Now, I only need to make it restart the server on system restart
  or new certificates.

Followed [the let's encrypt guide](https://letsencrypt.org/getting-started/) to
add tls, since .dev domains have to be https (who knew?). I'm using certbot,
which is running on
[snapd](https://snapcraft.io/docs/installing-snap-on-debian).

Should look into podman quadlets to make the container start again after a
reboot.

Didn't look at podman quadlets, but I added a service to systemd to start up the
server after a reboot. It should be put in
`/etc/systemd/system/podman-rust-score-tracker.service` and enabled with
`systemctl enable podman-rust-score-tracker`. Added the file to the repo.

Additionally turned on automatic updates and automatic reboots. The server
already had all the tools required and enabled (I followed
[https://wiki.debian.org/UnattendedUpgrades]), so it was just a matter of
updating the configuration in `/etc/apt/apt.conf.d/50unattended-upgrades`.

Podman service didn't start up after reboot, which is a known issue
(https://github.com/containers/podman/issues/22197), so I added a 'user level'
service to wait for network as a workaround.

Podman recommends using quadlets instead, so I've removed the services again.

Podman service still not starting up after reboot. They recommend using
quadlets, but those are only enabled in podman 4.4, and I'm on 4.3, and debian
11 doesn't have a more recent version, and upgrading to a newer one apparently
isn't simple (it's about upgrading the package repository) :sadface:. So now I'm
following this in podman 4.3:
https://docs.podman.io/en/v4.3/markdown/podman-generate-systemd.1.html. The
first time, it is also required to do
`sudo systemctl enable score-tracker-server`.

The tl;dr is: Make sure the server is running, and do

```
podman generate systemd rust_score_tracker_server --new > score-tracker-server.service
sudo mv score-tracker-server.service /etc/systemd/system/
sudo systemctl daemon-reload
```

Still not working. Will give up for now.

Tried to get `unattended-upgrades` to send me emails, but didn't get that to
work either. Enough for today.

Connect to a running docker container:

- `podman ps` to find the container id.
- `podman exec -it <id> bash`

Came across something recently. This blog post:
https://matduggan.com/replace-compose-with-quadlet/. I can apparently use
`loginctl enable-linger <username>` to make the user able to start services at
boot without logging in. Will try that.

Tried it and rebooting. Come on.

So the service is starting, but it's starting as root, not as <username>. And
that doesn't work.

To see the logs for systemd, use
`journalctl -xe -u score-tracker-server.service`.

IT'S FINALLY WORKINGGGGGGGG!!!!!

So! The magic thing to do was:

1. Follow the guide at
   https://docs.podman.io/en/v4.3/markdown/podman-generate-systemd.1.html.
   That's the version of podman I have, so that's the guide I should use.
2. Enable user lingering with `sudo loginctl enable-linger <username>`. This
   allows that user to start services after a reboot.
3. Start the service with the run-command above.
4. Generate a systemd file with
   `podman generate systemd rust_score_tracker_server --new > score-tracker-server.service`
   (the service needs to run).
5. `mv score-tracker-server.service ~/.config/systemd/user/`. May need to make
   the directories along the way.
6. `systemctl --user enable score-tracker-server.service`.
7. `systemctl --user start score-tracker-server.service`.

Just one more thing remains: I need to make it restart when there are new
certificates (and then at some point I should check what is necessary for me to
be able to update it with a new image). But this is a great start.

Done!
