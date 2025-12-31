// ESM Circular Dependency - Module A
import { getFromA } from './b.js';

export const valueFromA = 'A';
export function getFromB() {
    return getFromA();
}
