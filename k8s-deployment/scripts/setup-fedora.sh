#!/bin/bash
set -e

echo "=========================================="
echo "Minikube Setup for Fedora"
echo "=========================================="
echo ""

# Check if running as root
if [ "$EUID" -eq 0 ]; then 
   echo "Please do not run as root. This script will use sudo when needed."
   exit 1
fi

# Update system
echo "[1/6] Updating system packages..."
sudo dnf update -y

# Install dependencies
echo "[2/6] Installing dependencies..."
sudo dnf install -y curl wget conntrack

# Install kubectl
echo "[3/6] Installing kubectl..."
if ! command -v kubectl &> /dev/null; then
    KUBECTL_VERSION=$(curl -L -s https://dl.k8s.io/release/stable.txt)
    curl -LO "https://dl.k8s.io/release/${KUBECTL_VERSION}/bin/linux/amd64/kubectl"
    chmod +x kubectl
    sudo mv kubectl /usr/local/bin/
    echo "kubectl installed: $(kubectl version --client --short 2>/dev/null || kubectl version --client)"
else
    echo "kubectl already installed: $(kubectl version --client --short 2>/dev/null || kubectl version --client)"
fi

# Install container runtime (podman is default on Fedora)
echo "[4/6] Checking container runtime..."
if ! command -v podman &> /dev/null; then
    sudo dnf install -y podman
    echo "Podman installed"
else
    echo "Podman already installed: $(podman --version)"
fi

# Install minikube
echo "[5/6] Installing minikube..."
if ! command -v minikube &> /dev/null; then
    curl -LO https://storage.googleapis.com/minikube/releases/latest/minikube-linux-amd64
    chmod +x minikube-linux-amd64
    sudo mv minikube-linux-amd64 /usr/local/bin/minikube
    echo "minikube installed: $(minikube version --short)"
else
    echo "minikube already installed: $(minikube version --short)"
fi

# Start minikube with podman driver
echo "[6/6] Starting minikube..."
if minikube status &> /dev/null; then
    echo "minikube is already running"
    minikube status
else
    echo "Starting minikube with podman driver..."
    minikube start --driver=podman --cpus=4 --memory=8192 --disk-size=20g
    echo ""
    echo "minikube started successfully!"
fi

# Verify installation
echo ""
echo "=========================================="
echo "Verification"
echo "=========================================="
echo "kubectl version:"
kubectl version --client --short 2>/dev/null || kubectl version --client
echo ""
echo "minikube version:"
minikube version --short
echo ""
echo "minikube status:"
minikube status
echo ""
echo "Cluster info:"
kubectl cluster-info
echo ""

echo "=========================================="
echo "Setup Complete!"
echo "=========================================="
echo ""
echo "Next steps:"
echo "  1. Run ./deploy.sh to deploy the Ubuntu nodes and bastion"
echo "  2. Run ./verify.sh to check the deployment"
echo ""
