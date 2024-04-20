set -euxo pipefail

aws s3 sync s3://fehstatsim-staging s3://fehstatsim.fullyconcentrated.net --delete
aws cloudfront create-invalidation --distribution-id E1VV0KHMINNOB --paths "/*"