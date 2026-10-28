#!/bin/sh
# Unprivileged PostgreSQL development binaries from signed Debian packages.
set -eu
umask 077
task_tools=/workspace/setup-tools
task_apt=$task_tools/apt
task_pg=$task_tools/postgres
mkdir -p "$task_apt/lists/partial" "$task_apt/cache/archives/partial" "$task_apt/empty"
cat > "$task_apt/config" <<EOF
Dir::Etc::parts "$task_apt/empty";
Dir::Etc::main "-";
Dir::Etc::sourcelist "$task_apt/sources.list";
Dir::Etc::sourceparts "$task_apt/empty";
Dir::State::lists "$task_apt/lists";
Dir::Cache "$task_apt/cache";
APT::Sandbox::User "$(id -un)";
EOF
if [ -n "${HTTPS_PROXY:-}" ]; then
    # The managed environment proxy URL is a non-secret service endpoint.
    # Do not put credential-bearing proxy URLs in this generated configuration.
    case "$HTTPS_PROXY" in
        *'@'*) echo 'Credential-bearing proxy URL is unsupported' >&2; exit 1 ;;
    esac
    printf 'Acquire::https::Proxy "%s";\n' "$HTTPS_PROXY" >> "$task_apt/config"
fi
cat > "$task_apt/sources.list" <<'EOF'
deb [signed-by=/usr/share/keyrings/debian-archive-keyring.gpg] https://deb.debian.org/debian trixie main
EOF
export APT_CONFIG=$task_apt/config
apt-get update
task_download=$(mktemp -d "$task_apt/download.XXXXXX")
trap 'rm -rf "$task_download"' EXIT HUP INT TERM
(
    cd "$task_download"
    apt-get download postgresql-17 postgresql-client-17 libpq5 libllvm19 libicu76 libxml2 libxslt1.1 liblz4-1 libzstd1
)
mkdir -p "$task_pg"
for task_package in "$task_download"/*.deb; do
    dpkg-deb -x "$task_package" "$task_pg"
done
LD_LIBRARY_PATH=$task_pg/usr/lib/x86_64-linux-gnu${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH} \
    "$task_pg/usr/lib/postgresql/17/bin/postgres" --version
