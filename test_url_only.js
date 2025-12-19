const url = new URL("https://api.example.com:8080/data/users?limit=10&offset=0#details");
console.log("URL Test: " + url.href + " | Host: " + url.host + " | Protocol: " + url.protocol);
