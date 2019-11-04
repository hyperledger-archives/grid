interface Canopy {
  invokeRegisteredApp(domNode: Node): void;
}

interface Window {
  $CANOPY: Canopy;
}
