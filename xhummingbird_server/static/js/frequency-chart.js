var ctx = document.getElementById('frequency-chart').getContext('2d');

var labels = [];
for(var i = 0; i < 168; i++){
  labels.unshift(i);
}

var chart = new Chart(ctx, {
  type: 'bar',
  data: {
    labels: labels,
    datasets: [{
      label: 'Num of Event',
      data: data,
      backgroundColor: '#6acd7f',
    }]
  },
  options: {
    plugins: {
      legend: {
        display: false,
      }
    },
    scales: {
      y: {
        beginAtZero: true,
        title: {
          display: true,
          text: 'Num of events',
        },
        ticks: {
          stepSize: 1,
        }
      },
      x: {
        title: {
          display: true,
          text: 'Hours ago',
        }
      },
    },
    animation:false,
  }
});
