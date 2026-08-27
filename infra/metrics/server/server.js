const express = require('express');
const client = require('prom-client');

const app = express();
app.use(express.json());

// --- Prometheus setup ---
const register = new client.Registry();
client.collectDefaultMetrics({ register });

const emailEventsCounter = new client.Counter({
  name: 'resend_emails_total',
  help: 'Nombre d\'events email par type',
  labelNames: ['event'],
});
register.registerMetric(emailEventsCounter);

app.get('/metrics', async (req, res) => {
  res.set('Content-Type', register.contentType);
  res.end(await register.metrics());
});

app.get('/health', (req, res) => {
  res.status(200).send('ok');
});

// --- Webhook Resend ---
app.post('/webhooks/resend', (req, res) => {
  const event = req.body;

  console.log(JSON.stringify(event));

  const eventType = event.type.replace('email.', '');
  emailEventsCounter.inc({ event: eventType });

  res.status(200).send('ok');
});

app.listen(3001, () => {
  console.log('Serveur webhook démarré sur le port 3001');
});