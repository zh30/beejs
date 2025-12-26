// Async/await test
async function fetchData(): Promise<string> {
    return new Promise(resolve => {
        resolve("Data loaded!");
    });
}

(async () => {
    const data = await fetchData();
    console.log(data);
})();
