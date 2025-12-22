#!/bin/bash

echo "=========================================="
echo "SSH Connectivity Test"
echo "=========================================="
echo ""

if ! minikube status &> /dev/null; then
    echo "❌ ERROR: minikube is not running"
    exit 1
fi

MINIKUBE_IP=$(minikube ip)
echo "Minikube IP: $MINIKUBE_IP"
echo ""

# Test function
test_ssh_port() {
    local port=$1
    local name=$2
    
    echo -n "Testing $name (port $port)... "
    
    if timeout 5 bash -c "nc -z $MINIKUBE_IP $port" 2>/dev/null; then
        echo "✓ Port is open"
        return 0
    else
        echo "❌ Port is not accessible"
        return 1
    fi
}

# Test bastion
echo "[1/7] Testing SSH Bastion"
test_ssh_port 30222 "SSH Bastion"
echo ""

# Test individual nodes
echo "[2/7] Testing Individual Ubuntu Nodes"
for i in {1..6}; do
    PORT=$((30100 + i))
    test_ssh_port $PORT "Ubuntu Node $i"
done
echo ""

# Test load-balanced access
echo "[3/7] Testing Load-Balanced Access"
test_ssh_port 30022 "Ubuntu Nodes (load-balanced)"
echo ""

# Try actual SSH connection (password-based)
echo "[4/7] Testing Actual SSH Connection"
echo "Attempting SSH to node 1 (will prompt for password if accessible)"
echo "Password: testnode123"
echo ""
ssh -o StrictHostKeyChecking=no -o ConnectTimeout=5 -p 30101 root@$MINIKUBE_IP "echo '✓ SSH connection successful!'; hostname; uname -a" 2>/dev/null || echo "⚠ Could not establish SSH connection (try manually with password: testnode123)"
echo ""

# Test HTTP health endpoint
echo "[5/7] Testing HTTP Health Endpoints"
if curl -s -m 3 http://$MINIKUBE_IP:30080/health 2>/dev/null | grep -q "healthy"; then
    echo "✓ Health endpoint is responding"
    echo ""
    echo "Sample health response:"
    curl -s -m 3 http://$MINIKUBE_IP:30080/health 2>/dev/null | head -10
else
    echo "⚠ Health endpoint is not responding"
fi
echo ""

# Get pod IPs for bastion testing
echo "[6/7] Pod Information"
echo "Ubuntu node pods and their IPs:"
kubectl get pods -l app=ubuntu-test-node -o custom-columns=NAME:.metadata.name,IP:.status.podIP,STATUS:.status.phase
echo ""
echo "Bastion pod:"
kubectl get pod ssh-bastion -o custom-columns=NAME:.metadata.name,IP:.status.podIP,STATUS:.status.phase
echo ""

# Summary
echo "[7/7] Summary"
echo "=========================================="
echo ""
echo "Direct access from host machine:"
echo "  Bastion:  ssh -p 30222 root@$MINIKUBE_IP"
echo "  Node 1:   ssh -p 30101 root@$MINIKUBE_IP"
echo "  Node 2:   ssh -p 30102 root@$MINIKUBE_IP"
echo "  Node 3:   ssh -p 30103 root@$MINIKUBE_IP"
echo "  Node 4:   ssh -p 30104 root@$MINIKUBE_IP"
echo "  Node 5:   ssh -p 30105 root@$MINIKUBE_IP"
echo "  Node 6:   ssh -p 30106 root@$MINIKUBE_IP"
echo ""
echo "Password for Ubuntu nodes: testnode123"
echo ""
echo "For bastion access, ensure your SSH public key is in:"
echo "  manifests/03-ssh-bastion.yaml"
echo ""
echo "=========================================="
echo "Test Complete!"
echo "=========================================="
