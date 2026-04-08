#!/usr/bin/env node
/**
 * Apple OAuth Client Secret 생성 스크립트
 *
 * 사용법:
 *   node scripts/generate_apple_secret.js \
 *     --team-id=D22ZM93S77 \
 *     --key-id=YOUR_KEY_ID \
 *     --client-id=com.frank.web \
 *     --key-file=path/to/AuthKey_XXXXXXXX.p8
 *
 * 생성된 JWT를 Supabase Apple Provider의 "Secret Key (for OAuth)" 필드에 입력.
 * 유효기간: 6개월 (Apple 정책상 최대)
 */

import { createPrivateKey, createSign } from 'crypto';
import { readFileSync } from 'fs';

function parseArgs() {
	const args = {};
	for (const arg of process.argv.slice(2)) {
		const [key, value] = arg.replace(/^--/, '').split('=');
		args[key] = value;
	}
	return args;
}

function base64url(input) {
	const buf = typeof input === 'string' ? Buffer.from(input) : input;
	return buf.toString('base64').replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');
}

function generateAppleClientSecret({ teamId, keyId, clientId, privateKeyPem }) {
	const now = Math.floor(Date.now() / 1000);
	const exp = now + 15777000; // 6개월

	const header = base64url(JSON.stringify({ alg: 'ES256', kid: keyId }));
	const payload = base64url(
		JSON.stringify({
			iss: teamId,
			iat: now,
			exp,
			aud: 'https://appleid.apple.com',
			sub: clientId
		})
	);

	const signingInput = `${header}.${payload}`;
	const privateKey = createPrivateKey(privateKeyPem);
	const sign = createSign('SHA256');
	sign.update(signingInput);
	const rawSignature = sign.sign({ key: privateKey, dsaEncoding: 'ieee-p1363' });
	const signature = base64url(rawSignature);

	return `${signingInput}.${signature}`;
}

const args = parseArgs();
const required = ['team-id', 'key-id', 'client-id', 'key-file'];
const missing = required.filter((k) => !args[k]);

if (missing.length > 0) {
	console.error(`❌ 필수 인자 누락: ${missing.map((k) => `--${k}`).join(', ')}`);
	console.error('');
	console.error('사용법:');
	console.error(
		'  node scripts/generate_apple_secret.js --team-id=D22ZM93S77 --key-id=YOUR_KEY_ID --client-id=com.frank.web --key-file=path/to/AuthKey_XXXXXXXX.p8'
	);
	process.exit(1);
}

let privateKeyPem;
try {
	privateKeyPem = readFileSync(args['key-file'], 'utf8');
} catch (e) {
	console.error(`❌ .p8 파일을 읽을 수 없습니다: ${args['key-file']}`);
	process.exit(1);
}

const jwt = generateAppleClientSecret({
	teamId: args['team-id'],
	keyId: args['key-id'],
	clientId: args['client-id'],
	privateKeyPem
});

const exp = Math.floor(Date.now() / 1000) + 15777000;
const expDateObj = new Date(exp * 1000);
const expDateISO = expDateObj.toISOString().split('T')[0]; // YYYY-MM-DD (UTC 기준, 서버 비교 기준과 동일)

console.log('');
console.log('✅ Apple OAuth Client Secret 생성 완료');
console.log(`   만료일: ${expDateISO} (UTC)`);
console.log('');
console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
console.log('[Step 1] Supabase Apple Provider 설정');
console.log('  Supabase 대시보드 > Authentication > Providers > Apple');
console.log('  > Secret Key (for OAuth) 에 아래 JWT 붙여넣기:');
console.log('');
console.log(jwt);
console.log('');
console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
console.log('[Step 2] 배포 환경변수 설정');
console.log('  아래 환경변수를 Railway / Fly.io 대시보드에서 수동 설정:');
console.log('');
console.log(`  APPLE_CLIENT_SECRET_EXPIRES_AT=${expDateISO}`);
console.log('');
console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
console.log('[Step 3] 서버 재시작 후 로그 확인');
console.log('  배포 후 서버 로그에서 만료일 관련 메시지 확인:');
console.log('  - D-60 이상: 로그 없음 (정상)');
console.log('  - D-60 ~ D-31: INFO 레벨');
console.log('  - D-30 ~ D-8: WARN 레벨');
console.log('  - D-7 이하: ERROR 레벨');
console.log('');
