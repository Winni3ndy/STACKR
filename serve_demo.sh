#!/usr/bin/env bash
# Serve Stackr demo on local network so you can access from your phone

set -e

# Get local IP address
IP=$(ipconfig getifaddr en0 2>/dev/null || ipconfig getifaddr en1 2>/dev/null || echo "localhost")

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📱 Stackr Demo Server"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Demo is now available at:"
echo ""
echo "   On this computer:"
echo "   http://localhost:8000/demo.html"
echo ""
echo "   On your phone (same WiFi):"
echo "   http://$IP:8000/demo.html"
echo ""
echo "   USSD Simulator:"
echo "   http://$IP:8000/simulator/"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📝 Instructions:"
echo "1. Make sure Stackr server is running (cargo run)"
echo "2. Connect your phone to the same WiFi"
echo "3. Open the URL above on your phone"
echo "4. Use test account: +250781111111, PIN: 1234"
echo ""
echo "Press Ctrl+C to stop..."
echo ""

# Start simple HTTP server
python3 -m http.server 8000
