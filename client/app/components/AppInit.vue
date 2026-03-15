<template>
  <!-- Renderless component — sets up global subscription -->
</template>

<script setup lang="ts">
// This component is always rendered inside SpacetimeDBProvider,
// so useGlobalSubscription() has access to the SDK injection.
useGlobalSubscription()
</script>
