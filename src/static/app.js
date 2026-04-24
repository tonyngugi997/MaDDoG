async function callApi() {
    const output = document.getElementById('output');
    output.textContent = 'Loading...';
    
    try {
        const response = await fetch('/api/chat');
        const data = await response.json();
        output.textContent = JSON.stringify(data, null, 2);
    } catch (error) {
        output.textContent = `Error: ${error.message}`;
    }
}

async function callApiPost() {
    const output = document.getElementById('output');
    output.textContent = 'Loading...';
    
    try {
        const response = await fetch('/api/chat', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                message: 'Hello from browser!',
                timestamp: new Date().toISOString()
            })
        });
        const data = await response.json();
        output.textContent = JSON.stringify(data, null, 2);
    } catch (error) {
        output.textContent = `Error: ${error.message}`;
    }
}