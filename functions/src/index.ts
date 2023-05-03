import * as functions from 'firebase-functions';
import { defineString } from 'firebase-functions/params';
import * as admin from 'firebase-admin';
import { PubSub } from '@google-cloud/pubsub';
import { Storage } from '@google-cloud/storage';
import axios from 'axios';
import { Stream } from 'stream';
import { GoogleAuth } from 'google-auth-library';

admin.initializeApp();

const firestore = admin.firestore();
const jobs = firestore.collection('jobs');

const generationEndpoint = defineString('GENERATION_ENDPOINT');
const storageBucket = defineString('STORAGE_BUCKET');

const auth = new GoogleAuth();

export const generate = functions.https.onCall(async data => {
	const prompt = data?.prompt;
	if (typeof prompt !== 'string') {
		throw new functions.https.HttpsError(
			'invalid-argument', 'prompt is invalid or not present'
		);
	}

	const job = jobs.doc();
	await job.set({
		status: 'waiting',
		prompt,
	});

	await new PubSub().topic('generation').publishMessage({
		json: { prompt, jobId: job.id },
	});

	return { jobId: job.id };
});

export const generatePubSub = functions.pubsub
	.topic('generation')
	.onPublish(async message => {
		const { prompt, jobId } = message.json;

		try {
			const job = jobs.doc(jobId);
			job.update({ status: 'started' });

			const fileStream = await generateAndGetStream(prompt);
			const storageStream = new Storage()
				.bucket(storageBucket.value())
				.file(jobId + '.schematic')
				.createWriteStream();

			const pipe = fileStream.pipe(storageStream);
			pipe.on('finish', () => job.update({ status: 'finished' }));
			pipe.on('error', e => {
				throw e;
			});
		} catch (err) {
			await jobs.doc(jobId).update({ status: 'failed' });
			throw err;
		}
	});

/**
 * Makes a request to the configured generation endpoint and returns the response stream. If
 * Running in production, the request will be authenticated with IAM.
 */
async function generateAndGetStream(prompt: string): Promise<Stream> {
	const url = new URL(`/generate?prompt=${encodeURIComponent(prompt)}`, generationEndpoint.value());

	if (process.env.FUNCTIONS_EMULATOR === 'true') {
		return axios({ url: url.href, method: 'POST', responseType: 'stream'})
			.then(response => response.data);
	} else {
		const client = await auth.getIdTokenClient(generationEndpoint.value());

		return client.request({ url: url.href, method: 'POST', responseType: 'stream'})
			.then(response => response.data as Stream);
	}
}
