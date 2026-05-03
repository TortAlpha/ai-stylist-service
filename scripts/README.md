# sc deploy & backup scripts

Pull-based CD for the Raspberry Pi production host. The service-side workflow
(`.github/workflows/release.yml`) builds and pushes images to GHCR; the RPi
polls this repo's tags and rolls forward when a new `vX.Y.Z` tag appears.
A backup is taken before every roll.

## Files

| Script              | Role                                                                 |
|---------------------|----------------------------------------------------------------------|
| `deploy.sh`         | Orchestrator: detect new tag → backup → checkout → bump `.env` → up  |
| `backup.sh`         | Snapshot 3 Postgres + MinIO into `${SC_BACKUP_DIR}/<ts>.tar.gz`      |
| `restore.sh`        | Restore from a snapshot (interactive, asks for confirmation)         |
| `sc-deploy.service` | systemd unit that runs `deploy.sh`                                   |
| `sc-deploy.timer`   | systemd timer firing every 5 min                                     |

## Versions

`release.yml` builds images via `docker/metadata-action` with `{{version}}`,
which strips the leading `v`. So:

- Git tag in the repo: `v0.1.0`
- Docker image tag in GHCR: `0.1.0`
- `SC_VERSION` in `.env`: `0.1.0` (matches docker image tag)

`deploy.sh` strips the `v` from the git tag before comparing with `SC_VERSION`
and before writing it back to `.env`.

## RPi setup

Assumed layout on the host (current install):

```
/home/tortalpha/Code/sc/         ← git clone of this repo
  docker-compose.prod.yml
  .env                            ← SC_VERSION + creds (not committed)
  scripts/...
  backups/                        ← backup.sh output (created automatically)
```

One-time install:

```bash
cd /home/tortalpha/Code/sc

# 1. Make scripts executable
chmod +x scripts/*.sh

# 2. Install systemd unit + timer
sudo cp scripts/sc-deploy.service /etc/systemd/system/
sudo cp scripts/sc-deploy.timer   /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now sc-deploy.timer

# 3. (Optional) Trigger once to verify
sudo systemctl start sc-deploy.service
journalctl -u sc-deploy.service -n 100 --no-pager
```

If the host user / clone path are different from the defaults, edit
`User=`, `Group=`, `Environment=SC_REPO_DIR=`, `Environment=SC_BACKUP_DIR=`
and `ExecStart=` in `sc-deploy.service` before installing.

## How it triggers

1. CI builds an image when you push a `vX.Y.Z` tag (`release.yml`).
2. Within ~5 min the timer fires `deploy.sh` on the RPi.
3. `deploy.sh` runs `git fetch --tags`, finds the latest semver tag, strips
   the `v`, and compares with `SC_VERSION` in `.env`.
4. If different → `backup.sh` → `git checkout <tag>` → bump `.env` →
   `docker compose pull && up -d`.
5. If equal → silent exit.

## Manual operations

```bash
# Force a backup right now (no deploy)
SC_TAG="manual_$(date -u +%Y%m%d_%H%M%S)" \
  /home/tortalpha/Code/sc/scripts/backup.sh

# List backups
ls -lh /home/tortalpha/Code/sc/backups

# Restore from a specific snapshot
/home/tortalpha/Code/sc/scripts/restore.sh \
  /home/tortalpha/Code/sc/backups/20260503_120000_pre_v0.2.0.tar.gz

# Roll back to a previous tag without restoring data
cd /home/tortalpha/Code/sc
sed -i 's|^SC_VERSION=.*|SC_VERSION=0.1.0|' .env
git checkout v0.1.0
docker compose -f docker-compose.prod.yml pull
docker compose -f docker-compose.prod.yml up -d --remove-orphans
```

## Tunables

Override via the systemd `Environment=` lines or the shell:

| Var                  | Default                              | Purpose                                                   |
|----------------------|--------------------------------------|-----------------------------------------------------------|
| `SC_REPO_DIR`        | `/home/tortalpha/Code/sc`            | Where the git clone lives                                 |
| `SC_BACKUP_DIR`      | `/home/tortalpha/Code/sc/backups`    | Where snapshots are written                               |
| `SC_RETENTION_DAYS`  | `30`                                 | Prune `.tar.gz` older than N days (newest is always kept) |
| `COMPOSE_FILE`       | `docker-compose.prod.yml`            | Compose file relative to repo root                        |

## Caveats

- DB migrations run automatically inside `sc-product-service` on startup
  (`sqlx::migrate!`). The backup taken right before the roll is the rollback
  artefact if a migration goes sideways.
- `deploy.sh` does `git checkout <tag>` — if you have local edits in the
  live repo, the checkout will fail and the deploy will abort. Don't edit
  files in the live clone; treat it as read-only.
- Backups land on the same SD card as the host. If the SD dies, backups die
  with it. Off-site (e.g. Cloudflare R2) can be added later if needed —
  another `tar.gz` target in `backup.sh` or a daily `rclone copy` cron.
