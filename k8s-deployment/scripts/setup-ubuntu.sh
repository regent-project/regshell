#!/bin/bash
set -e

echo "=========================================="
echo "Minikube Setup for Ubuntu"
echo "=========================================="
echo ""

# Check if running as root
if [ "$EUID" -eq 0 ]; then 
   echo "Please do not run as root. This script will use sudo when needed."
   exit 1
fi

# Update system
echo "[1/7] Updating system packages..."
sudo apt update
sudo apt upgrade -y

# Install dependencies
echo "[2/7] Installing dependencies..."
sudo apt install -y curl wget apt-transport-https ca-certificates gnupg lsb-release conntrack

# Install Docker (preferred for Ubuntu)
echo "[3/7] Installing Docker..."
if ! command -v docker &> /dev/null; then
    # Add Docker's official GPG key
    sudo install -m 0755 -d /etc/apt/keyrings
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
    sudo chmod a+r /etc/apt/keyrings/docker.gpg
    
    # Add Docker repository
    echo \
      "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
      $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
    
    # Install Docker Engine
    sudo apt update
    sudo apt install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
    
    # Add current user to docker group
    sudo usermod -aG docker $USER
    echo "Docker installed. You may need to log out and back in for group changes to take effect."
    echo "Or run: newgrp docker"
else
    echo "Docker already installed: $(docker --version)"
fi

# Install kubectl
echo "[4/7] Installing kubectl..."
if ! command -v kubectl &> /dev/null; then
    KUBECTL_VERSION=$(curl -L -s https://dl.k8s.io/release/stable.txt)
    curl -LO "https://dl.k8s.io/release/${KUBECTL_VERSION}/bin/linux/amd64/kubectl"
    chmod +x kubectl
    sudo mv kubectl /usr/local/bin/
    echo "kubectl installed: $(kubectl version --client --short 2>/dev/null || kubectl version --client)"
else
    echo "kubectl already installed: $(kubectl version --client --short 2>/dev/null || kubectl version --client)"
fi

# Install minikube
echo "[5/7] Installing minikube..."
if ! command -v minikube &> /dev/null; then
    curl -LO https://storage.googleapis.com/minikube/releases/latest/minikube-linux-amd64
    chmod +x minikube-linux-amd64
    sudo mv minikube-linux-amd64 /usr/local/bin/minikube
    echo "minikube installed: $(minikube version --short)"
else
    echo "minikube already installed: $(minikube version --short)"
fi

# Ensure docker is running
echo "[6/7] Starting Docker service..."
sudo systemctl enable docker
sudo systemctl start docker

# Start minikube with docker driver
echo "[7/7] Starting minikube..."
if minikube status &> /dev/null; then
    echo "minikube is already running"
    minikube status
else
    echo "Starting minikube with docker driver..."
    # Use newgrp to apply docker group in current session if needed
    if groups | grep -q docker; then
        minikube start --driver=docker --cpus=4 --memory=8192 --disk-size=20g
    else
        echo "Applying docker group membership..."
        sg docker -c "minikube start --driver=docker --cpus=4 --memory=8192 --disk-size=20g"
    fi
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
echo "IMPORTANT: If you just installed Docker, you may need to:"
echo "  1. Log out and back in, OR"
echo "  2. Run: newgrp docker"
echo ""
echo "Next steps:"
echo "  1. Run ./deploy.sh to deploy the Ubuntu nodes and bastion"
echo "  2. Run ./verify.sh to check the deployment"
echo ""
