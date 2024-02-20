import{r as u}from"./index.NEDEFKed.js";var x={exports:{}},d={};/**
 * @license React
 * react-jsx-runtime.production.min.js
 *
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */var p=u,b=Symbol.for("react.element"),h=Symbol.for("react.fragment"),g=Object.prototype.hasOwnProperty,v=p.__SECRET_INTERNALS_DO_NOT_USE_OR_YOU_WILL_BE_FIRED.ReactCurrentOwner,w={key:!0,ref:!0,__self:!0,__source:!0};function m(e,r,o){var n,a={},i=null,c=null;o!==void 0&&(i=""+o),r.key!==void 0&&(i=""+r.key),r.ref!==void 0&&(c=r.ref);for(n in r)g.call(r,n)&&!w.hasOwnProperty(n)&&(a[n]=r[n]);if(e&&e.defaultProps)for(n in r=e.defaultProps,r)a[n]===void 0&&(a[n]=r[n]);return{$$typeof:b,type:e,key:i,ref:c,props:a,_owner:v.current}}d.Fragment=h;d.jsx=m;d.jsxs=m;x.exports=d;var t=x.exports;function _({title:e,children:r,className:o}){return t.jsxs("div",{className:`
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
    ${o}
    `,children:[t.jsx("div",{className:`\r
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
        `,children:"➖⏹️❌"}),t.jsxs("div",{className:`\r
            sm:overflow-y-auto\r
            scrollbar-thin\r
            scrollbar-track-base-background-color\r
            scrollbar-thumb-box-color-standard`,children:[t.jsx("div",{className:`\r
            font-headerf\r
            text-2xl\r
            md:text-3xl\r
            lg:text-4xl\r
        `,children:e}),t.jsx("div",{children:r})]})]})}var l=(e=>(e[e.standard=0]="standard",e[e.trans=1]="trans",e[e.nostalgia=2]="nostalgia",e[e.ace=3]="ace",e[e.bi=4]="bi",e[e.werwolvdark=5]="werwolvdark",e))(l||{});function y(){return s(),t.jsx("div",{children:t.jsx("a",{href:"#",onClick:e=>{e.preventDefault();let r=s();const o=r<f?r+1:0;localStorage.setItem("theme",o.toString()),console.log(`Theme: ${l[o]}`),location.reload()},children:t.jsx(_,{title:"",children:t.jsxs("div",{className:"content-center p-3 text-center",children:[t.jsx("div",{className:`
                  font-headerf
                  text-lg
                  md:text-xl
                  lg:text-2xl
                  bg-base-text-color/25
                  rounded-lg
                  p-3
                `,children:"Change Theme"}),t.jsx("div",{className:"font-light text-base-text-color/40 mb-[-12px]",children:`${l[s()]}`})]})})})})}const f=5;function k(){for(let e=0;e<=f;e++)document.documentElement.classList.remove(l[e]);document.documentElement.classList.add(l[s()])}function s(){let e=null,r=0;return typeof window<"u"&&(e=localStorage.getItem("theme")),typeof e=="number"?r=e:typeof e=="string"?r=parseInt(e):r=0,console.log(`Theme: ${l[r]}`),r}export{y as T,k as l};
