import * as functions from 'firebase-functions';
import { defineString } from 'firebase-functions/params';
import * as admin from 'firebase-admin';
import { PubSub } from '@google-cloud/pubsub';
import { Storage } from '@google-cloud/storage';
import axios, { AxiosResponse } from 'axios';
import { Stream } from 'stream';

admin.initializeApp();

const firestore = admin.firestore();
const jobs = firestore.collection('jobs');

const generationEndpoint = defineString('GENERATION_ENDPOINT');
const storageBucket = defineString('STORAGE_BUCKET');

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

			const response: AxiosResponse<Stream> = await axios({
				url: generationEndpoint.value() + `?prompt=${encodeURIComponent(prompt)}`,
				method: 'POST',
				responseType: 'stream',
			});

			const storage = new Storage()
				.bucket(storageBucket.value())
				.file(jobId + '.schematic');

			const stream = response.data.pipe(storage.createWriteStream());
			stream.on('finish', () => job.update({ status: 'finished' }));
			stream.on('error', e => {
				throw e;
			});
		} catch (err) {
			jobs.doc(jobId).update({ status: 'failed' });
			throw err;
		}
	});
