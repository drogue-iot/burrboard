// Note: This is a temporary access token and needs to be refreshed every few minutes
const accessToken = "eyJhbGciOiJSUzI1NiIsInR5cCIgOiAiSldUIiwia2lkIiA6ICJKR2VRRG01Szlpc1l3emFSTmdXME4zVU91UUlGWkFfZVFFc0QxREdNUzY0In0.eyJleHAiOjE2NDMwMzQ5NjUsImlhdCI6MTY0MzAzNDY2NSwiYXV0aF90aW1lIjoxNjQyMDg5NDY4LCJqdGkiOiI1NjQwNmEwMS0wZmYxLTQ1Y2ItOWUyZC1kNzJmZDljZjE0MjEiLCJpc3MiOiJodHRwczovL3Nzby1kcm9ndWUtZGV2LmFwcHMud29uZGVyZnVsLmlvdC1wbGF5Z3JvdW5kLm9yZy9hdXRoL3JlYWxtcy9kcm9ndWUiLCJhdWQiOlsiZHJvZ3VlIiwiYWNjb3VudCJdLCJzdWIiOiJmNzlkZDUyMC03MDdmLTQ0NGItYmI1MC01NTNmODE0MzVjZGMiLCJ0eXAiOiJCZWFyZXIiLCJhenAiOiJkcm9ndWUiLCJzZXNzaW9uX3N0YXRlIjoiYzg0MTRmMmQtODAzZC00YjBhLTkzNzMtYjkxZmZmMDAxZjQ4IiwiYWNyIjoiMCIsImFsbG93ZWQtb3JpZ2lucyI6WyJodHRwOi8vbG9jYWxob3N0OioiLCJodHRwczovL2NvbnNvbGUtZHJvZ3VlLWRldi5hcHBzLndvbmRlcmZ1bC5pb3QtcGxheWdyb3VuZC5vcmciXSwicmVhbG1fYWNjZXNzIjp7InJvbGVzIjpbImRyb2d1ZS11c2VyIiwib2ZmbGluZV9hY2Nlc3MiLCJ1bWFfYXV0aG9yaXphdGlvbiIsImRlZmF1bHQtcm9sZXMtZHJvZ3VlIl19LCJyZXNvdXJjZV9hY2Nlc3MiOnsiYWNjb3VudCI6eyJyb2xlcyI6WyJtYW5hZ2UtYWNjb3VudCIsIm1hbmFnZS1hY2NvdW50LWxpbmtzIiwidmlldy1wcm9maWxlIl19fSwic2NvcGUiOiJwcm9maWxlIGVtYWlsIG9mZmxpbmVfYWNjZXNzIiwic2lkIjoiYzg0MTRmMmQtODAzZC00YjBhLTkzNzMtYjkxZmZmMDAxZjQ4IiwiZW1haWxfdmVyaWZpZWQiOnRydWUsIm5hbWUiOiJKZW5zIFJlaW1hbm4iLCJwcmVmZXJyZWRfdXNlcm5hbWUiOiJjdHJvbiIsImdpdmVuX25hbWUiOiJKZW5zIiwiZmFtaWx5X25hbWUiOiJSZWltYW5uIiwiZW1haWwiOiJjdHJvbkBkZW50cmFzc2kuZGUifQ.iyJMi6WxOT9MUFKjdwweCs0utb9tIknjh4StLAniZPABU3Gh2szgOiJC2fIaP1oDfdFMg_FXsM_sGLJqnR1HBB6qPorieY17QKHV5U5exgznmynE9dC12YVznYH4wo4X8IabHYhlvGDCUXgyypBKfsEHxA2lO9uK7jQ1W01iRN24Q9bhj6KCudgtyXwyhhLlW1rRsBj90Lgjkc54u735FV2Iz7DysALVu2qAsqv9szMJIj9kXAtqLkPO6aUwYKW-u1eJBxvEO0VDqHZcvqxx0I5ilS6mtJ5khAJOSzzvqMclueA9bSex4bnEr-luWii5REwByyxyF4ZZOiuv1ah68g";

let client;

function connect() {
    setState(null);
    client = new WebSocket("wss://ditto-drogue-dev.apps.wonderful.iot-playground.org/ws/2?access_token=" + encodeURIComponent(accessToken));
    client.addEventListener('open', function(event) {
        console.log("Connected: ", event);
        client.send("START-SEND-EVENTS?extraFields=features/accelerometer");
    });
    client.addEventListener('message', function(event) {
        // console.log("Message: ", event);
        if (event.type !== "message") {
            return;
        }
        if (event.data === "START-SEND-EVENTS:ACK") {
            state.connected = true;
            return;
        }
        try {
            console.log(event.data);
            const data = JSON.parse(event.data);
            // when using WebSockets (with extraFields), fetch from the extra section
            const accel = data.extra.features.accelerometer.properties;
            setState(accel);
        }
        catch(e) {}

    });
    client.addEventListener('error', function(event) {
        state.connected = false;
        console.log("Error: ", event);
    });
    client.addEventListener('close', function(event) {
        state.connected = false;
        console.log("Closed: ", event);
    });
}
