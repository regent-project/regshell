#!/bin/bash
# All-in-one script for testing deployment on Ubuntu VM
# This script can be run on a fresh Ubuntu installation

set -e

echo "=========================================="
echo "Ubuntu VM Test - Complete Setup"
echo "=========================================="
echo ""
echo "This script will:"
echo "  1. Install all prerequisites"
echo "  2. Setup minikube"
echo "  3. Deploy the Ubuntu test environment"
echo "  4. Verify the deployment"
echo ""
read -p "Continue? (y/N) " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Cancelled."
    exit 0
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

echo ""
echo "=========================================="
echo "Step 1: System Prerequisites"
echo "=========================================="
echo ""

# Update and install basic tools
echo "Updating system..."
sudo apt update
sudo apt upgrade -y

echo "Installing prerequisites..."
sudo apt install -y \
    curl wget git \
    apt-transport-https ca-certificates \
    gnupg lsb-release conntrack \
    net-tools netcat-openbsd

echo ""
echo "=========================================="
echo "Step 2: Installing Docker"
echo "=========================================="
echo ""

if command -v docker &> /dev/null; then
    echo "Docker already installed: $(docker --version)"
else
    echo "Installing Docker..."
    
    # Add Docker's official GPG key
    sudo install -m 0755 -d /etc/apt/keyrings
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
    sudo chmod a+r /etc/apt/keyrings/docker.gpg
    
    # Add repository
    echo \
      "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
      $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
    
    # Install Docker
    sudo apt update
    sudo apt install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
    
    # Add user to docker group
    sudo usermod -aG docker $USER
    
    echo "Docker installed successfully!"
    echo "Applying group membership..."
    
    # Start docker
    sudo systemctl enable docker
    sudo systemctl start docker
fi

echo ""
echo "=========================================="
echo "Step 3: Installing kubectl"
echo "=========================================="
echo ""

if command -v kubectl &> /dev/null; then
    echo "kubectl already installed"
else
    echo "Installing kubectl..."
    KUBECTL_VERSION=$(curl -L -s https://dl.k8s.io/release/stable.txt)
    curl -LO "https://dl.k8s.io/release/${KUBECTL_VERSION}/bin/linux/amd64/kubectl"
    chmod +x kubectl
    sudo mv kubectl /usr/local/bin/
    echo "kubectl installed: $(kubectl version --client)"
fi

echo ""
echo "=========================================="
echo "Step 4: Installing minikube"
echo "=========================================="
echo ""

if command -v minikube &> /dev/null; then
    echo "minikube already installed"
else
    echo "Installing minikube..."
    curl -LO https://storage.googleapis.com/minikube/releases/latest/minikube-linux-amd64
    chmod +x minikube-linux-amd64
    sudo mv minikube-linux-amd64 /usr/local/bin/minikube
    echo "minikube installed: $(minikube version --short)"
fi

echo ""
echo "=========================================="
echo "Step 5: Starting minikube"
echo "=========================================="
echo ""

if minikube status &> /dev/null 2>&1; then
    echo "minikube is already running"
else
    echo "Starting minikube..."
    echo "This may take a few minutes..."
    
    # Use sg to run with docker group if needed
    if groups | grep -q docker; then
        minikube start --driver=docker --cpus=4 --memory=8192 --disk-size=20g
    else
        sg docker -c "minikube start --driver=docker --cpus=4 --memory=8192 --disk-size=20g"
    fi
    
    echo "minikube started!"
fi

echo ""
echo "Minikube status:"
minikube status

echo ""
echo "Minikube IP: $(minikube ip)"

echo ""
echo "=========================================="
echo "Step 6: Deploying Ubuntu Test Environment"
echo "=========================================="
echo ""

echo "Applying Kubernetes manifests..."

kubectl apply -f manifests/01-ubuntu-nodes-deployment.yaml
kubectl apply -f manifests/02-ubuntu-nodes-services.yaml
kubectl apply -f manifests/03-ssh-bastion.yaml
kubectl apply -f manifests/04-individual-node-services.yaml

echo ""
echo "Waiting for pods to be ready..."
echo "This may take 3-5 minutes as containers are downloaded and started..."

# Wait for deployment
kubectl wait --for=condition=available --timeout=300s deployment/ubuntu-test-nodes || true

# Wait for bastion
kubectl wait --for=condition=ready --timeout=300s pod/ssh-bastion || true

echo ""
echo "=========================================="
echo "Step 7: Verification"
echo "=========================================="
echo ""

./scripts/verify.sh

echo ""
echo "=========================================="
echo "Deployment Complete!"
echo "=========================================="
echo ""

MINIKUBE_IP=$(minikube ip)

echo "Your Ubuntu test environment is ready!"
echo ""
echo "Access Information:"
echo "  Minikube IP: $MINIKUBE_IP"
echo ""
echo "SSH Access (password: testnode123):"
echo "  Node 1: ssh -p 30101 root@$MINIKUBE_IP"
echo "  Node 2: ssh -p 30102 root@$MINIKUBE_IP"
echo "  Node 3: ssh -p 30103 root@$MINIKUBE_IP"
echo "  Node 4: ssh -p 30104 root@$MINIKUBE_IP"
echo "  Node 5: ssh -p 30105 root@$MINIKUBE_IP"
echo "  Node 6: ssh -p 30106 root@$MINIKUBE_IP"
echo ""
echo "Test SSH connectivity:"
echo "  ./scripts/test-ssh.sh"
echo ""
echo "For regshell integration:"
echo "  1. Update inventory: sed -i 's/192\\.168\\.49\\.2/$MINIKUBE_IP/g' inventory/test-inventory.yaml"
echo "  2. Start console: regshell -i inventory/test-inventory.yaml"
echo ""
echo "See QUICKSTART.md and README.md for more information."
echo ""
echo "=========================================="
echo "Setup Complete! 🚀"
echo "=========================================="
