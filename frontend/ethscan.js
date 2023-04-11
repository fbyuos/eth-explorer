let first = 1;

(async function () {
    const chartCanvas_transaction = document.getElementById("transaction-chart");
    const chartavgCanvas_gas = document.getElementById("gas-avg-chart");
    const chartCanvas_gas = document.getElementById("gas-chart");
    const chartCanvas_txsvalue = document.getElementById("txs-value-chart");
    const chartCanvas_txsvaluemin = document.getElementById("txs-value-min-chart");
    let chart;
    let chart2;
    let chart3;
    let chart4;
    let chart5;

 

    async function fetchData(url) {
        const response = await fetch(url);
        const data = await response.json();
        return data;
    }

    async function fetchHistoricData() {
        return await fetchData("http://127.0.0.1:8080/historic-data");
    }

    async function displayData() {
        const transactions = await fetchData("http://127.0.0.1:8080/transactions");
        const blocks = await fetchData("http://127.0.0.1:8080/blocks");

        document.getElementById("transactionData").innerText = JSON.stringify(transactions, null, 2);
        document.getElementById("blockData").innerText = JSON.stringify(blocks, null, 2);
    }

    function createTransactionChart(historicData) {
        const blockNumbers = historicData.map(block => block.number);
        const transactionCounts = historicData.map(block => block.transaction_number);

        chart = new Chart(chartCanvas_transaction, {
            type: 'line',
            data: {
                labels: blockNumbers,
                datasets: [{
                    label: 'Transactions per Block',
                    data: transactionCounts,
                    backgroundColor: 'rgba(75, 192, 192, 0.2)',
                    borderColor: 'rgba(75, 192, 192, 1)',
                    borderWidth: 1
                }]
            },
            options: {
                scales: {
                    y: {
                        beginAtZero: true
                    }
                }
            }
        });
    }

    function createAvgGasChart(historicData) {
        const blockNumbers = historicData.map(block => block.number);
        const gasPrices = historicData.map(block => {
            // Calculate the average gas price of all transactions within the block
            const totalGasPrice = block.transactions.reduce((sum, transaction) => sum + Number(transaction.gas_price), 0);
            const avgGasPrice = block.transactions.length > 0 ? totalGasPrice / block.transactions.length : 0;
            return avgGasPrice;
        });
    
        chart2 = new Chart(chartavgCanvas_gas, {
            type: 'line',
            data: {
                labels: blockNumbers,
                datasets: [{
                    label: 'Average Gas Price per Block',
                    data: gasPrices,
                    backgroundColor: 'rgba(75, 192, 192, 0.2)',
                    borderColor: 'rgba(75, 192, 192, 1)',
                    borderWidth: 1
                }]
            },
            options: {
                scales: {
                    y: {
                        beginAtZero: true
                    }
                }
            }
        });
    }

    function createGasChart(historicData) {
        const blockNumbers = historicData.map(block => block.number);
        const totalGasPrices = historicData.map(block => {
            // Calculate the total gas price of all transactions within the block
            const totalGasPrice = block.transactions.reduce((sum, transaction) => sum + Number(transaction.gas_price), 0);
            return totalGasPrice;
        });
    
        chart3 = new Chart(chartCanvas_gas, {
            type: 'line',
            data: {
                labels: blockNumbers,
                datasets: [{
                    label: 'Total Gas Price per Block',
                    data: totalGasPrices,
                    backgroundColor: 'rgba(75, 192, 192, 0.2)',
                    borderColor: 'rgba(75, 192, 192, 1)',
                    borderWidth: 1
                }]
            },
            options: {
                scales: {
                    y: {
                        beginAtZero: true
                    }
                }
            }
        });
    }
    
    function createTxsValueChart(historicData) {
        const blockNumbers = historicData.map(block => block.number);
        const totalValuePrices = historicData.map(block => {
            // Calculate the total value of all transactions within the block
            const totalGasPrice = block.transactions.reduce((sum, transaction) => sum + Number(transaction.value), 0);
            return totalGasPrice;
        });
    
        chart4 = new Chart(chartCanvas_txsvalue, {
            type: 'line',
            data: {
                labels: blockNumbers,
                datasets: [{
                    label: 'Total Value per Block',
                    data: totalValuePrices,
                    backgroundColor: 'rgba(75, 192, 192, 0.2)',
                    borderColor: 'rgba(75, 192, 192, 1)',
                    borderWidth: 1
                }]
            },
            options: {
                scales: {
                    y: {
                        beginAtZero: true
                    }
                }
            }
        });
    }
    

    
    function createTransactionValueChart(historicData) {
        const minutelyTransactionSums = {};
    
        for (const block of historicData) {
            for (const transaction of block.transactions) {
                // Convert the hexadecimal timestamp to a Date object
                const timestamp = parseInt(block.timestamp, 16) * 1000;
                const date = new Date(timestamp);
    
                // Create a string representation of the date with minute precision
                const dateString = `${date.getFullYear()}-${date.getMonth() + 1}-${date.getDate()} ${date.getHours()}:${date.getMinutes()}`;
    
                // Calculate the sum of transaction values per minute
                if (minutelyTransactionSums[dateString] === undefined) {
                    minutelyTransactionSums[dateString] = 0;
                }
                minutelyTransactionSums[dateString] += Number(transaction.value);
            }
        }
    
        const chartLabels = Object.keys(minutelyTransactionSums);
        const chartData = chartLabels.map(dateString => minutelyTransactionSums[dateString]);
    
        chart5 = new Chart(chartCanvas_txsvaluemin, {
            type: 'line',
            data: {
                labels: chartLabels,
                datasets: [{
                    label: 'Transaction Value per Minute',
                    data: chartData,
                    backgroundColor: 'rgba(75, 192, 192, 0.2)',
                    borderColor: 'rgba(75, 192, 192, 1)',
                    borderWidth: 1
                }]
            },
            options: {
                scales: {
                    y: {
                        beginAtZero: true
                    }
                }
            }
        });
    }
    
    function toggleSpinner(visibility) {
        const spinner = document.getElementById("loadingSpinner");
        spinner.style.display = visibility ? "block" : "none";
    }

    async function updateData() {
        if (first == 1) {
            toggleSpinner(true); // Show the spinner
            first = 0;
        }


        await displayData();

        const historicData = await fetchHistoricData();
        if (chart) {
            chart.destroy();
        }
        createTransactionChart(historicData);
        createAvgGasChart(historicData);
        createGasChart(historicData);
        createTxsValueChart(historicData);
        createTransactionValueChart(historicData);

        toggleSpinner(false); // Hide the spinner
    }

    // Initial data fetch and chart rendering
    updateData();

    // Refresh data every 30 seconds (30000 milliseconds)
    setInterval(updateData, 30000);
})();




    /* PER DAY
    
    function createTransactionValueChart(historicData) {
        const dailyTransactionSums = {};
    
        for (const block of historicData) {
            for (const transaction of block.transactions) {
                // Convert the hexadecimal timestamp to a Date object
                const timestamp = parseInt(block.timestamp, 16) * 1000;
                const date = new Date(timestamp);
    
                // Create a string representation of the date (ignoring time)
                const dateString = `${date.getFullYear()}-${date.getMonth() + 1}-${date.getDate()}`;
    
                // Calculate the sum of transaction values per day
                if (dailyTransactionSums[dateString] === undefined) {
                    dailyTransactionSums[dateString] = 0;
                }
                dailyTransactionSums[dateString] += Number(transaction.value);
            }
        }
    
        const chartLabels = Object.keys(dailyTransactionSums);
        const chartData = chartLabels.map(dateString => dailyTransactionSums[dateString]);
    
        chart5 = new Chart(chartCanvas_txsvalueday, {
            type: 'line',
            data: {
                labels: chartLabels,
                datasets: [{
                    label: 'Transaction Value per Day',
                    data: chartData,
                    backgroundColor: 'rgba(75, 192, 192, 0.2)',
                    borderColor: 'rgba(75, 192, 192, 1)',
                    borderWidth: 1
                }]
            },
            options: {
                scales: {
                    y: {
                        beginAtZero: true
                    }
                }
            }
        });
    }*/



    /* PER HOUR
    
    
    function createTransactionValueChart(historicData) {
        const hourlyTransactionSums = {};
    
        for (const block of historicData) {
            for (const transaction of block.transactions) {
                // Convert the hexadecimal timestamp to a Date object
                const timestamp = parseInt(block.timestamp, 16) * 1000;
                const date = new Date(timestamp);
    
                // Create a string representation of the date with hour precision
                const dateString = `${date.getFullYear()}-${date.getMonth() + 1}-${date.getDate()} ${date.getHours()}:00`;
    
                // Calculate the sum of transaction values per hour
                if (hourlyTransactionSums[dateString] === undefined) {
                    hourlyTransactionSums[dateString] = 0;
                }
                hourlyTransactionSums[dateString] += Number(transaction.value);
            }
        }
    
        const chartLabels = Object.keys(hourlyTransactionSums);
        const chartData = chartLabels.map(dateString => hourlyTransactionSums[dateString]);
    
        chart5 = new Chart(chartCanvas_txsvaluehour, {
            type: 'line',
            data: {
                labels: chartLabels,
                datasets: [{
                    label: 'Transaction Value per Hour',
                    data: chartData,
                    backgroundColor: 'rgba(75, 192, 192, 0.2)',
                    borderColor: 'rgba(75, 192, 192, 1)',
                    borderWidth: 1
                }]
            },
            options: {
                scales: {
                    y: {
                        beginAtZero: true
                    }
                }
            }
        });
    }
    
    */