#!/bin/bash

# Rust Media Downloader - Development Startup Script

echo "🚀 Starting Rust Media Downloader Web Application"
echo "=================================================="
echo ""

# Check if backend is running
if lsof -Pi :8080 -sTCP:LISTEN -t >/dev/null ; then
    echo "⚠️  Backend already running on port 8080"
else
    echo "📦 Starting Backend API..."
    cd backend
    cargo run &
    BACKEND_PID=$!
    cd ..
    echo "✅ Backend started (PID: $BACKEND_PID)"
fi

# Wait a bit for backend to start
sleep 2

# Check if frontend is running
if lsof -Pi :5173 -sTCP:LISTEN -t >/dev/null ; then
    echo "⚠️  Frontend already running on port 5173"
else
    echo "🎨 Starting Frontend..."
    cd frontend
    
    # Install dependencies if needed
    if [ ! -d "node_modules" ]; then
        echo "📥 Installing frontend dependencies..."
        npm install
    fi
    
    npm run dev &
    FRONTEND_PID=$!
    cd ..
    echo "✅ Frontend started (PID: $FRONTEND_PID)"
fi

echo ""
echo "=================================================="
echo "✨ Application is ready!"
echo ""
echo "📍 Frontend: http://localhost:5173"
echo "📍 Backend API: http://localhost:8080"
echo "📍 Health Check: http://localhost:8080/health"
echo ""
echo "Press Ctrl+C to stop all services"
echo "=================================================="

# Wait for user interrupt
wait
