// ESM Circular Dependency - Module B
import { valueFromA } from './a.js';

export const valueFromB = 'B';
export function getFromA() {
    return valueFromA;
}
