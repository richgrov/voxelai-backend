import * as functions from 'firebase-functions';
import * as admin from 'firebase-admin';

admin.initializeApp();
const firestore = admin.firestore();
const jobs = firestore.collection('jobs');

export const generate = functions.https.onCall(async data => {
	const prompt = data.prompt;

	const job = jobs.doc();
	await job.set({
		status: 'waiting',
		prompt,
	});

	return { jobId: job.id };
});
