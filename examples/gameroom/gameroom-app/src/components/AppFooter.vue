<!--
Copyright 2019 Cargill Incorporated

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
-->

<template>
  <footer>
    <span>
      <a href="/">Gameroom</a>
      - An example application built on Splinter. Copyright 2019 Cargill Incorporated. Licensed
      under the Apache License, Version 2.0.
    </span>
  </footer>
</template>

<script>
export default {};
</script>

<style lang="scss" scoped>
footer {
  display: flex;
  background-color: $color-base;
  color: $color-text-tertiary;
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  font-size: .75em;
  align-items: center;
  padding: .5em
}
</style>
