// QR Code Scanner using jsQR library
// We'll load jsQR from CDN in the HTML

class QRScanner {
    constructor(videoElement, canvasElement) {
        this.video = videoElement;
        this.canvas = canvasElement;
        this.context = this.canvas.getContext('2d');
        this.scanning = false;
        this.stream = null;
    }

    async startScanning() {
        try {
            console.log('Requesting camera access...');
            
            // Request camera access
            this.stream = await navigator.mediaDevices.getUserMedia({
                video: { 
                    facingMode: 'environment', // Use back camera if available
                    width: { ideal: 1280 },
                    height: { ideal: 720 }
                }
            });
            
            console.log('Camera access granted');
            
            this.video.srcObject = this.stream;
            
            // Wait for video to be ready
            await new Promise((resolve, reject) => {
                this.video.onloadedmetadata = () => {
                    console.log('Video metadata loaded');
                    resolve();
                };
                this.video.onerror = (error) => {
                    console.error('Video error:', error);
                    reject(error);
                };
                
                // Timeout after 10 seconds
                setTimeout(() => {
                    reject(new Error('Video loading timeout'));
                }, 10000);
            });
            
            await this.video.play();
            console.log('Video started playing');
            
            this.scanning = true;
            this.scanFrame();
            
            return { success: true };
        } catch (error) {
            console.error('Error accessing camera:', error);
            
            let errorMessage = 'Failed to access camera';
            if (error.name === 'NotAllowedError') {
                errorMessage = '摄像头权限被拒绝，请允许访问摄像头';
            } else if (error.name === 'NotFoundError') {
                errorMessage = '未找到摄像头设备';
            } else if (error.name === 'NotSupportedError') {
                errorMessage = '浏览器不支持摄像头功能';
            } else if (error.message) {
                errorMessage = error.message;
            }
            
            return { success: false, error: errorMessage };
        }
    }

    stopScanning() {
        this.scanning = false;
        if (this.stream) {
            this.stream.getTracks().forEach(track => track.stop());
            this.stream = null;
        }
        this.video.srcObject = null;
    }

    scanFrame() {
        if (!this.scanning) return;

        if (this.video.readyState === this.video.HAVE_ENOUGH_DATA) {
            this.canvas.height = this.video.videoHeight;
            this.canvas.width = this.video.videoWidth;
            
            this.context.drawImage(this.video, 0, 0, this.canvas.width, this.canvas.height);
            
            const imageData = this.context.getImageData(0, 0, this.canvas.width, this.canvas.height);
            
            // Check if jsQR is available
            if (typeof jsQR !== 'undefined') {
                const code = jsQR(imageData.data, imageData.width, imageData.height);
                
                if (code) {
                    this.onQRCodeDetected(code.data);
                    return;
                }
            }
        }
        
        requestAnimationFrame(() => this.scanFrame());
    }

    onQRCodeDetected(data) {
        // This will be called from Rust/Dioxus
        // console.log('QR Code detected:', data);
        
        // Dispatch custom event
        const event = new CustomEvent('qr-code-detected', { 
            detail: { data: data }
        });
        window.dispatchEvent(event);
    }
}

// Global QR scanner instance
let qrScanner = null;

// Expose functions to be called from Rust
window.initQRScanner = function(videoId, canvasId) {
    console.log('Initializing QR scanner with:', videoId, canvasId);
    
    const video = document.getElementById(videoId);
    const canvas = document.getElementById(canvasId);
    
    if (!video || !canvas) {
        console.error('Video or canvas element not found', { 
            video: !!video, 
            canvas: !!canvas,
            videoId: videoId,
            canvasId: canvasId
        });
        return false;
    }
    
    console.log('Elements found, creating QR scanner');
    qrScanner = new QRScanner(video, canvas);
    return true;
};

window.startQRScanning = async function() {
    if (!qrScanner) {
        console.error('QR Scanner not initialized');
        return { success: false, error: 'QR Scanner not initialized' };
    }
    
    return await qrScanner.startScanning();
};

window.stopQRScanning = function() {
    if (qrScanner) {
        qrScanner.stopScanning();
    }
};

// Get camera zoom capabilities
window.getCameraZoomCapabilities = async function() {
    if (!qrScanner || !qrScanner.stream) {
        return { min: 1.0, max: 1.0, step: 1.0 };
    }
    
    try {
        const track = qrScanner.stream.getVideoTracks()[0];
        if (!track) {
            return { min: 1.0, max: 1.0, step: 1.0 };
        }
        
        const capabilities = track.getCapabilities();
        if (capabilities.zoom) {
            return {
                min: capabilities.zoom.min || 1.0,
                max: capabilities.zoom.max || 1.0,
                step: Math.max(capabilities.zoom.step || 1.0, 1.0) // Use 1.0 as minimum step
            };
        } else {
            console.log('Camera does not support zoom');
            return { min: 1.0, max: 1.0, step: 1.0 };
        }
    } catch (error) {
        console.error('Error getting zoom capabilities:', error);
        return { min: 1.0, max: 1.0, step: 1.0 };
    }
};

// Set camera zoom level
window.setCameraZoom = async function(zoomLevel) {
    if (!qrScanner || !qrScanner.stream) {
        console.error('QR Scanner or stream not available');
        return false;
    }
    
    try {
        const track = qrScanner.stream.getVideoTracks()[0];
        if (!track) {
            console.error('No video track available');
            return false;
        }
        
        const capabilities = track.getCapabilities();
        if (!capabilities.zoom) {
            console.log('Camera does not support zoom');
            return false;
        }
        
        // Clamp zoom level to supported range
        const minZoom = capabilities.zoom.min || 1.0;
        const maxZoom = capabilities.zoom.max || 1.0;
        const clampedZoom = Math.max(minZoom, Math.min(maxZoom, zoomLevel));
        
        await track.applyConstraints({
            advanced: [{
                zoom: clampedZoom
            }]
        });
        
        console.log(`Zoom set to: ${clampedZoom}`);
        return true;
    } catch (error) {
        console.error('Error setting zoom:', error);
        return false;
    }
};
