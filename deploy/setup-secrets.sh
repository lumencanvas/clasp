#!/bin/bash
# Set up GitHub repo secrets for the deploy-relay workflow.
# Run from repo root: ./deploy/setup-secrets.sh
# Requires: gh CLI authenticated, SSH key for Droplet
set -e

echo "Setting DROPLET_IP..."
gh secret set DROPLET_IP --body "157.230.84.254"

echo "Setting DROPLET_SSH_KEY..."
echo "Paste your Droplet SSH private key (then Ctrl+D):"
gh secret set DROPLET_SSH_KEY

echo "Done. Secrets configured for deploy-relay workflow."
echo "GITHUB_TOKEN is automatic — no setup needed for GHCR push."
