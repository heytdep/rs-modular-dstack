#!/bin/bash

# Exit script on error
set -e

# Check arguments
if [ "$#" -lt 2 ]; then
  echo "Usage: $0 <CLUSTER> <SECRET> [PUBKEY]"
  exit 1
fi

CLUSTER="$1"
SECRET="$2"
PUBKEY="${3:-}"  # this is optional

generate_newyork_yml() {
  echo "Generating newyork.yml with CLUSTER=$CLUSTER..."
  cat <<EOF > newyork.yml
apiVersion: v1
kind: Pod
metadata:
  name: new-york-pod
  labels:
    app: new-york
spec:
  containers:
    - name: new-york-container
      image: xycloo/new-york-image:latest
      env:
        - name: CLUSTER
          value: "$CLUSTER"
EOF
  if [ -n "$PUBKEY" ]; then
    echo "        - name: PUBKEY" >> newyork.yml
    echo "          value: \"$PUBKEY\"" >> newyork.yml
  fi

  cat <<EOF >> newyork.yml
    - name: ping-host-container
      image: xycloo/ping-host-image:latest
  hostNetwork: true
EOF

  echo "newyork.yml created successfully."
}

# Build new-york host on host machine
build_and_run_rust() {
  echo "Building Rust application..."
  pushd new-york
  cargo build --release
  popd

  echo "Running Rust application..."
  CLUSTER="$CLUSTER" SECRET="$SECRET" ./target/release/host &
}

# Upload to flashbox!
send_curl_requests() {
  echo "Sending pod to flashbox..."

  # Do we need this at all? Doing just to be sure but the pod yml should be inferring the vars.
  echo "CLUSTER=$CLUSTER" > env
  echo "PUBKEY=$PUBKEY" >> env

  # Upload
  curl -X POST -F "pod.yaml=@newyork.yml" -F "env=@env" http://localhost:24070/upload
  echo "Pod uploaded."

  # Start
  curl -X POST http://localhost:24070/start
  echo "Pod started successfully."
}

generate_newyork_yml
build_and_run_rust
send_curl_requests

echo "All tasks completed successfully!"
