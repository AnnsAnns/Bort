import{r as p}from"./index.NEDEFKed.js";var m={exports:{}},s={};/**
 * @license React
 * react-jsx-runtime.production.min.js
 *
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */var h=p,b=Symbol.for("react.element"),g=Symbol.for("react.fragment"),v=Object.prototype.hasOwnProperty,w=h.__SECRET_INTERNALS_DO_NOT_USE_OR_YOU_WILL_BE_FIRED.ReactCurrentOwner,_={key:!0,ref:!0,__self:!0,__source:!0};function f(e,r,l){var t,a={},c=null,i=null;l!==void 0&&(c=""+l),r.key!==void 0&&(c=""+r.key),r.ref!==void 0&&(i=r.ref);for(t in r)v.call(r,t)&&!_.hasOwnProperty(t)&&(a[t]=r[t]);if(e&&e.defaultProps)for(t in r=e.defaultProps,r)a[t]===void 0&&(a[t]=r[t]);return{$$typeof:b,type:e,key:c,ref:i,props:a,_owner:w.current}}s.Fragment=g;s.jsx=f;s.jsxs=f;m.exports=s;var n=m.exports;function j({title:e,children:r,className:l}){return n.jsxs("div",{className:`
    bg-base-background-color
    ring-box-color-standard
    ring-4
    break-words
    overflow-hidden
    shadow-square
    shadow-box-color-standard/50
    min-h-[120px]
    min-w-[120px]
    flex flex-col
    m-1
    px-3
    pb-3
    sm:max-h-notfullscreen
    text-base-text-color
    ${l}
    `,children:[n.jsx("div",{className:`\r
            bg-box-color-standard\r
            mx-[-12px]\r
            min-w-max\r
            min-h-min\r
            items-end\r
            text-left\r
            text-end\r
            flex\r
            justify-end\r
            align-middle\r
            p-1\r
            pb-2\r
            gap-1\r
            text-base-background-color\r
        `,children:"➖⏹️❌"}),n.jsxs("div",{className:`\r
            sm:overflow-y-auto\r
            scrollbar-thin\r
            scrollbar-track-base-background-color\r
            scrollbar-thumb-box-color-standard`,children:[n.jsx("div",{className:`\r
            font-headerf\r
            text-2xl\r
            md:text-3xl\r
            lg:text-4xl\r
        `,children:e}),n.jsx("div",{children:r})]})]})}var o=(e=>(e[e.standard=0]="standard",e[e.trans=1]="trans",e[e.nostalgia=2]="nostalgia",e[e.ace=3]="ace",e[e.bi=4]="bi",e[e.werwolvdark=5]="werwolvdark",e))(o||{});function k(){let e=d();return x(),n.jsx("div",{children:n.jsx("a",{href:"#",onClick:r=>{r.preventDefault(),e=d();const l=e<u?e+1:0;localStorage.setItem("theme",l.toString()),console.log(`Theme: ${o[l]}`),x();const t=document.querySelector(".theme-name");t&&(t.textContent=o[l])},children:n.jsx(j,{title:"",children:n.jsxs("div",{className:"content-center p-3 text-center",children:[n.jsx("div",{className:`
                  font-headerf
                  text-lg
                  md:text-xl
                  lg:text-2xl
                  bg-base-text-color/25
                  rounded-lg
                  p-3
                `,children:"Change Theme"}),n.jsx("div",{className:"theme-name font-light text-base-text-color/40 mb-[-12px]",children:`${o[e]}`})]})})})})}const u=5;function x(){console.log("Loading theme");const e=d();document.documentElement.classList.add(o[e]);for(let r=0;r<=u;r++)o[r]!==o[e]&&(console.log(`Removing ${o[r]}`),document.documentElement.classList.remove(o[r]))}function d(){let e=null,r=0;return typeof window<"u"&&(e=localStorage.getItem("theme")),typeof e=="number"?r=e:typeof e=="string"?r=parseInt(e):r=0,console.log(`Theme: ${o[r]}`),r}export{k as T,x as l};
