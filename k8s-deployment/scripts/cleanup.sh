#!/bin/bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MANIFEST_DIR="$(cd "$SCRIPT_DIR/../manifests" && pwd)"

echo "=========================================="
echo "Cleanup Kubernetes Resources"
echo "=========================================="
echo ""

read -p "This will delete all deployed resources. Continue? (y/N) " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Cleanup cancelled."
    exit 0
fi

echo ""
echo "Deleting resources..."
echo ""

# Delete in reverse order
echo "[1/4] Deleting individual node services..."
kubectl delete -f "$MANIFEST_DIR/04-individual-node-services.yaml" --ignore-not-found=true

echo "[2/4] Deleting SSH bastion..."
kubectl delete -f "$MANIFEST_DIR/03-ssh-bastion.yaml" --ignore-not-found=true

echo "[3/4] Deleting Ubuntu nodes services..."
kubectl delete -f "$MANIFEST_DIR/02-ubuntu-nodes-services.yaml" --ignore-not-found=true

echo "[4/4] Deleting Ubuntu nodes deployment..."
kubectl delete -f "$MANIFEST_DIR/01-ubuntu-nodes-deployment.yaml" --ignore-not-found=true

echo ""
echo "Waiting for pods to terminate..."
kubectl wait --for=delete pod -l app=ubuntu-test-node --timeout=60s 2>/dev/null || true
kubectl wait --for=delete pod ssh-bastion --timeout=60s 2>/dev/null || true

echo ""
echo "=========================================="
echo "Cleanup Complete!"
echo "=========================================="
echo ""
echo "Current pods:"
kubectl get pods
echo ""
echo "Current services:"
kubectl get svc | grep -E "NAME|ubuntu|bastion" || echo "No ubuntu/bastion services found"
echo ""
echo "To completely remove minikube:"
echo "  minikube delete"
echo ""
