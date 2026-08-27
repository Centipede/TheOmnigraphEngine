export function isTypingElement(el: Element | null): boolean {
  if (!el) return false;
  const tagName = el.tagName.toUpperCase();
  // Elements that genuinely need to capture all keyboard input
  if (['INPUT', 'TEXTAREA', 'SELECT'].includes(tagName)) return true;
  if (['SL-INPUT', 'SL-TEXTAREA', 'SL-SELECT'].includes(tagName)) return true;
  return (el as HTMLElement).isContentEditable;
}

export function isTypingTarget(target: EventTarget | null): boolean {
  const el = target as Element;
  if (isTypingElement(el)) return true;
  const shadowActiveElement = el?.shadowRoot?.activeElement;
  return isTypingElement(shadowActiveElement || null);
}
