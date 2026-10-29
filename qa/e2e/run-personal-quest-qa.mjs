// Uses the same isolated database/session lifecycle as the training regression.
process.env.WW_QA_JOURNEY = 'personal-quest';
await import('./run-training-qa.mjs');
