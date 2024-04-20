set -euxo pipefail

cd egui_frontend
trunk build --release
aws s3 sync dist/ s3://fehstatsim-staging --delete