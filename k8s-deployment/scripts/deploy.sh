#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MANIFEST_DIR="$(cd "$SCRIPT_DIR/../manifests" && pwd)"

echo "=========================================="
echo "Deploying Ubuntu Test Environment"
echo "=========================================="
echo ""

# Check if minikube is running
if ! minikube status &> /dev/null; then
    echo "ERROR: minikube is not running!"
    echo "Please run the appropriate setup script first:"
    echo "  - ./setup-fedora.sh (for Fedora)"
    echo "  - ./setup-ubuntu.sh (for Ubuntu)"
    exit 1
fi

echo "Minikube IP: $(minikube ip)"
echo ""

# Deploy manifests in order
echo "[1/4] Deploying Ubuntu nodes deployment..."
kubectl apply -f "$MANIFEST_DIR/01-ubuntu-nodes-deployment.yaml"

echo "[2/4] Deploying Ubuntu nodes services..."
kubectl apply -f "$MANIFEST_DIR/02-ubuntu-nodes-services.yaml"

echo "[3/4] Deploying SSH bastion..."
kubectl apply -f "$MANIFEST_DIR/03-ssh-bastion.yaml"

echo "[4/4] Deploying individual node services..."
kubectl apply -f "$MANIFEST_DIR/04-individual-node-services.yaml"

echo ""
echo "=========================================="
echo "Deployment Complete!"
echo "=========================================="
echo ""
echo "Waiting for pods to be ready..."
echo "This may take a few minutes as containers download and start..."
echo ""

# Wait for deployment
kubectl wait --for=condition=available --timeout=300s deployment/ubuntu-test-nodes 2>/dev/null || true

# Wait for bastion
kubectl wait --for=condition=ready --timeout=300s pod/ssh-bastion 2>/dev/null || true

echo ""
echo "Current pod status:"
kubectl get pods
echo ""

echo "Services:"
kubectl get svc | grep -E "NAME|ubuntu|bastion"
echo ""

MINIKUBE_IP=$(minikube ip)
echo "=========================================="
echo "Access Information"
echo "=========================================="
echo ""
echo "SSH Bastion:"
echo "  ssh -p 30222 root@$MINIKUBE_IP"
echo ""
echo "Individual Ubuntu Nodes:"
echo "  ssh -p 30101 root@$MINIKUBE_IP  # node-1"
echo "  ssh -p 30102 root@$MINIKUBE_IP  # node-2"
echo "  ssh -p 30103 root@$MINIKUBE_IP  # node-3"
echo "  ssh -p 30104 root@$MINIKUBE_IP  # node-4"
echo "  ssh -p 30105 root@$MINIKUBE_IP  # node-5"
echo "  ssh -p 30106 root@$MINIKUBE_IP  # node-6"
echo ""
echo "Ubuntu Nodes Password: testnode123"
echo ""
echo "Next steps:"
echo "  1. Run ./verify.sh to verify the deployment"
echo "  2. Run ./test-ssh.sh to test SSH connectivity"
echo ""
